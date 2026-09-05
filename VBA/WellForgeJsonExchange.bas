Attribute VB_Name = "WellForgeJsonExchange"
Option Explicit

' WellForge desktop JSON exchange.  This module deliberately uses late-bound
' COM objects so an imported .bas file does not add workbook references and
' remains usable on offline VBA7 installations.

Private Const MODULE_SOURCE As String = "WellForgeJsonExchange"
Private Const SCHEMA_VERSION As String = "1.0.0"
Private Const MAP_SHEET As String = "Exchange Map"
Private Const STATE_SHEET As String = "Exchange State"
Private Const BUFFER_SHEET As String = "Exchange Buffer"
Private Const BUFFER_PAYLOAD_CELL As String = "B5"
Private Const BUFFER_STATUS_CELL As String = "B7"
Private Const BUFFER_DIAGNOSTICS_CELL As String = "B8"
Private Const FIRST_MAP_ROW As Long = 6
Private Const FIRST_STATE_ROW As Long = 6
Private Const ERR_BASE As Long = vbObjectError + 7400

Private WF_EXCHANGE_INJECT_WRITE_FAILURE As Boolean
Private WF_EXCHANGE_LAST_ROLLBACK_VERIFIED As Boolean

' symbol|accepted dimensions|multiplier|offset.  Decimal text is parsed with
' Val, not CDbl, so the private constant table is locale-independent.
Private Const UNIT_TABLE_1 As String = _
    "1|unitless|1|0;m|length,diameter|1|0;ft|length,diameter|0.3048|0;" & _
    "km|length|1000|0;mm|diameter,length|0.001|0;in|diameter,length|0.0254|0;" & _
    "m2|area|1|0;ft2|area|0.09290304|0;in2|area|0.00064516|0;cm2|area|0.0001|0;" & _
    "m3|volume|1|0;bbl|volume|0.158987294928|0;gal|volume|0.003785411784|0;L|volume|0.001|0;"
Private Const UNIT_TABLE_2 As String = _
    "m3/s|flowRate|1|0;L/s|flowRate|0.001|0;L/min|flowRate|0.000016666666666666667|0;" & _
    "gpm|flowRate|0.0000630901964|0;kg/m3|density|1|0;ppg|density|119.826427316|0;" & _
    "lb/ft3|density|16.01846337396|0;N|force|1|0;lbf|force|4.4482301|0;" & _
    "klbf|force|4448.2301|0;kN|force|1000|0;Pa|pressure,stress|1|0;" & _
    "kPa|pressure|1000|0;GPa|pressure,stress|1000000000|0;" & _
    "psi|pressure,stress|6894.757293168|0;Mpsi|pressure,stress|6894757293.168|0;bar|pressure|100000|0;"
Private Const UNIT_TABLE_3 As String = _
    "N*m|torque|1|0;N-m|torque|1|0;kN*m|torque|1000|0;kN-m|torque|1000|0;" & _
    "ft-lbf|torque|1.3558179483314|0;MPa|pressure,stress|1000000|0;ksi|stress|6894757.293168|0;" & _
    "rad|angle|1|0;deg|angle|0.017453292519943295|0;m/s|speed|1|0;" & _
    "ft/min|speed|0.00508|0;m/min|speed|0.016666666666666666|0;" & _
    "rad/m|angularGradient|1|0;deg/100ft|angularGradient|0.0005729577951308232|0;" & _
    "deg/30m|angularGradient|0.0005817764173314432|0;"
Private Const UNIT_TABLE_4 As String = _
    "Pa*s|viscosity|1|0;Pa*s^n|rheologyConsistency|1|0;1/Pa|compressibility|1|0;" & _
    "cP|viscosity|0.001|0;Hz|frequency|1|0;" & _
    "rad/s|rotationalSpeed|1|0;rpm|rotationalSpeed|0.10471975511965977|0;" & _
    "d|date|86400|0;K|temperature|1|0;C|temperature|1|273.15;" & _
    "F|temperature|0.5555555555555556|255.3722222222222"

Public Sub WellForge_LoadJson()
    Dim oldCalculation As XlCalculation
    Dim oldEnableEvents As Boolean
    Dim oldScreenUpdating As Boolean
    Dim selectedPath As String
    Dim payloadText As String
    Dim failureNumber As Long
    Dim failureText As String

    oldCalculation = Application.Calculation
    oldEnableEvents = Application.EnableEvents
    oldScreenUpdating = Application.ScreenUpdating
    On Error GoTo Failed

    With Application.FileDialog(msoFileDialogFilePicker)
        .Title = "Load WellForge JSON"
        .AllowMultiSelect = False
        .Filters.Clear
        .Filters.Add "JSON files", "*.json"
        If .Show <> -1 Then GoTo Cleanup
        selectedPath = .SelectedItems(1)
    End With

    Application.Calculation = xlCalculationManual
    Application.EnableEvents = False
    Application.ScreenUpdating = False
    payloadText = ReadUtf8File(selectedPath)
    ImportPayloadText payloadText
    WellForge_CalculateAll False
    WriteExchangeBufferStatus "Imported", "Loaded " & selectedPath
    GoTo Cleanup

Failed:
    failureNumber = Err.Number
    failureText = Err.Description
    On Error Resume Next
    WriteExchangeBufferStatus "Error", failureText
    On Error GoTo 0

Cleanup:
    On Error Resume Next
    Application.Calculation = oldCalculation
    Application.EnableEvents = oldEnableEvents
    Application.ScreenUpdating = oldScreenUpdating
    On Error GoTo 0
    If failureNumber <> 0 Then MsgBox failureText, vbExclamation, "WellForge JSON import"
End Sub

Public Sub WellForge_SaveJson()
    Dim oldCalculation As XlCalculation
    Dim oldEnableEvents As Boolean
    Dim oldScreenUpdating As Boolean
    Dim selectedPath As String
    Dim payloadText As String
    Dim failureNumber As Long
    Dim failureText As String

    oldCalculation = Application.Calculation
    oldEnableEvents = Application.EnableEvents
    oldScreenUpdating = Application.ScreenUpdating
    On Error GoTo Failed

    With Application.FileDialog(msoFileDialogSaveAs)
        .Title = "Save WellForge JSON"
        .InitialFileName = ThisWorkbook.Path & Application.PathSeparator & "wellforge-exchange.json"
        If .Show <> -1 Then GoTo Cleanup
        selectedPath = .SelectedItems(1)
    End With
    If LCase$(Right$(selectedPath, 5)) <> ".json" Then selectedPath = selectedPath & ".json"

    Application.Calculation = xlCalculationManual
    Application.EnableEvents = False
    Application.ScreenUpdating = False
    payloadText = ExportPayloadText()
    AtomicWriteUtf8 selectedPath, payloadText
    WriteExchangeBufferStatus "Exported", "Saved " & selectedPath
    GoTo Cleanup

Failed:
    failureNumber = Err.Number
    failureText = Err.Description
    On Error Resume Next
    WriteExchangeBufferStatus "Error", failureText
    On Error GoTo 0

Cleanup:
    On Error Resume Next
    Application.Calculation = oldCalculation
    Application.EnableEvents = oldEnableEvents
    Application.ScreenUpdating = oldScreenUpdating
    On Error GoTo 0
    If failureNumber <> 0 Then MsgBox failureText, vbExclamation, "WellForge JSON export"
End Sub

Public Sub WellForge_ExchangeRollbackSelfTest()
    Dim failureNumber As Long, failureText As String, injectedFailure As Long
    Dim payloadText As String
    On Error GoTo Failed

    payloadText = ReadExchangeBuffer()
    WF_EXCHANGE_LAST_ROLLBACK_VERIFIED = False
    WF_EXCHANGE_INJECT_WRITE_FAILURE = True
    On Error Resume Next
    ImportPayloadText payloadText
    injectedFailure = Err.Number
    Err.Clear
    On Error GoTo Failed
    WF_EXCHANGE_INJECT_WRITE_FAILURE = False
    If injectedFailure = 0 Then RaiseExchangeError "Injected exchange write failure did not occur"
    If Not WF_EXCHANGE_LAST_ROLLBACK_VERIFIED Then RaiseExchangeError "Exchange rollback equality verification failed"
    Exit Sub

Failed:
    failureNumber = Err.Number
    failureText = Err.Description
    WF_EXCHANGE_INJECT_WRITE_FAILURE = False
    Err.Raise failureNumber, "WellForge_ExchangeRollbackSelfTest", failureText
End Sub

Public Sub WellForge_ValidateExchange()
    Dim payload As Variant
    Dim diagnostics As String
    On Error GoTo Failed

    payload = Empty
    If IsJsonContainerStart(ReadExchangeBuffer()) Then
        Set payload = JsonParse(ReadExchangeBuffer())
    Else
        payload = JsonParse(ReadExchangeBuffer())
    End If
    ValidatePayload payload, diagnostics
    WriteExchangeBufferStatus "Valid", IIf(Len(diagnostics) = 0, "Schema " & SCHEMA_VERSION, diagnostics)
    Exit Sub

Failed:
    WriteExchangeBufferStatus "Invalid", Err.Description
End Sub

Public Function JsonParse(ByVal Text As String) As Variant
    Dim Cursor As Long
    Dim parsed As Variant

    Cursor = 1
    SkipWhitespace Cursor, Text
    If Cursor > Len(Text) Then RaiseExchangeError "JSON payload is empty"
    If IsJsonContainerStartAt(Cursor, Text) Then
        Set parsed = ParseValue(Cursor, Text)
        Set JsonParse = parsed
    Else
        parsed = ParseValue(Cursor, Text)
        JsonParse = parsed
    End If
    SkipWhitespace Cursor, Text
    If Cursor <= Len(Text) Then RaiseExchangeError "JSON contains trailing tokens at character " & CStr(Cursor)
End Function

Private Function ParseValue(ByRef Cursor As Long, ByVal Text As String) As Variant
    Dim ch As String

    SkipWhitespace Cursor, Text
    If Cursor > Len(Text) Then RaiseExchangeError "Unexpected end of JSON"
    ch = Mid$(Text, Cursor, 1)
    Select Case ch
        Case "{"
            Set ParseValue = ParseObject(Cursor, Text)
        Case "["
            Set ParseValue = ParseArray(Cursor, Text)
        Case """"
            ParseValue = ParseString(Cursor, Text)
        Case "t"
            ExpectLiteral Cursor, Text, "true"
            ParseValue = True
        Case "f"
            ExpectLiteral Cursor, Text, "false"
            ParseValue = False
        Case "n"
            ExpectLiteral Cursor, Text, "null"
            ParseValue = Null
        Case "-", "0" To "9"
            ParseValue = ParseNumber(Cursor, Text)
        Case Else
            RaiseExchangeError "Unexpected JSON token at character " & CStr(Cursor)
    End Select
End Function

Private Function ParseObject(ByRef Cursor As Long, ByVal Text As String) As Object
    Dim result As Object
    Dim key As String
    Dim memberValue As Variant
    Dim ch As String

    Set result = CreateObject("Scripting.Dictionary")
    result.CompareMode = vbBinaryCompare
    Cursor = Cursor + 1
    SkipWhitespace Cursor, Text
    If CharacterAt(Cursor, Text) = "}" Then
        Cursor = Cursor + 1
        Set ParseObject = result
        Exit Function
    End If

    Do
        SkipWhitespace Cursor, Text
        If CharacterAt(Cursor, Text) <> """" Then RaiseExchangeError "Object key must be a JSON string"
        key = ParseString(Cursor, Text)
        If result.Exists(key) Then RaiseExchangeError "Duplicate JSON object key: " & key
        SkipWhitespace Cursor, Text
        If CharacterAt(Cursor, Text) <> ":" Then RaiseExchangeError "Expected ':' after object key"
        Cursor = Cursor + 1
        SkipWhitespace Cursor, Text
        If IsJsonContainerStartAt(Cursor, Text) Then
            Set memberValue = ParseValue(Cursor, Text)
        Else
            memberValue = ParseValue(Cursor, Text)
        End If
        result.Add key, memberValue
        SkipWhitespace Cursor, Text
        ch = CharacterAt(Cursor, Text)
        If ch = "}" Then
            Cursor = Cursor + 1
            Exit Do
        End If
        If ch <> "," Then RaiseExchangeError "Expected ',' or '}' in object"
        Cursor = Cursor + 1
    Loop
    Set ParseObject = result
End Function

Private Function ParseArray(ByRef Cursor As Long, ByVal Text As String) As Collection
    Dim result As Collection
    Dim item As Variant
    Dim ch As String

    Set result = New Collection
    Cursor = Cursor + 1
    SkipWhitespace Cursor, Text
    If CharacterAt(Cursor, Text) = "]" Then
        Cursor = Cursor + 1
        Set ParseArray = result
        Exit Function
    End If

    Do
        SkipWhitespace Cursor, Text
        If IsJsonContainerStartAt(Cursor, Text) Then
            Set item = ParseValue(Cursor, Text)
        Else
            item = ParseValue(Cursor, Text)
        End If
        result.Add item
        SkipWhitespace Cursor, Text
        ch = CharacterAt(Cursor, Text)
        If ch = "]" Then
            Cursor = Cursor + 1
            Exit Do
        End If
        If ch <> "," Then RaiseExchangeError "Expected ',' or ']' in array"
        Cursor = Cursor + 1
    Loop
    Set ParseArray = result
End Function

Private Function ParseString(ByRef Cursor As Long, ByVal Text As String) As String
    Dim result As String
    Dim ch As String
    Dim escapeCode As String
    Dim highSurrogate As Long
    Dim lowSurrogate As Long

    Cursor = Cursor + 1
    Do While Cursor <= Len(Text)
        ch = Mid$(Text, Cursor, 1)
        Cursor = Cursor + 1
        If ch = """" Then
            ParseString = result
            Exit Function
        End If
        If AscW(ch) >= 0 And AscW(ch) < 32 Then RaiseExchangeError "Unescaped control character in JSON string"
        If ch <> "\" Then
            result = result & ch
        Else
            If Cursor > Len(Text) Then RaiseExchangeError "Incomplete JSON escape"
            escapeCode = Mid$(Text, Cursor, 1)
            Cursor = Cursor + 1
            Select Case escapeCode
                Case """": result = result & """"
                Case "\": result = result & "\"
                Case "/": result = result & "/"
                Case "b": result = result & ChrW$(8)
                Case "f": result = result & ChrW$(12)
                Case "n": result = result & vbLf
                Case "r": result = result & vbCr
                Case "t": result = result & vbTab
                Case "u"
                    highSurrogate = ParseHexCodeUnit(Cursor, Text)
                    If highSurrogate >= &HD800 And highSurrogate <= &HDBFF Then
                        If Mid$(Text, Cursor, 2) <> "\u" Then RaiseExchangeError "High surrogate is not followed by a low surrogate"
                        Cursor = Cursor + 2
                        lowSurrogate = ParseHexCodeUnit(Cursor, Text)
                        If lowSurrogate < &HDC00 Or lowSurrogate > &HDFFF Then RaiseExchangeError "Invalid low surrogate"
                        result = result & CodeUnitText(highSurrogate) & CodeUnitText(lowSurrogate)
                    ElseIf highSurrogate >= &HDC00 And highSurrogate <= &HDFFF Then
                        RaiseExchangeError "Unpaired low surrogate"
                    Else
                        result = result & CodeUnitText(highSurrogate)
                    End If
                Case Else
                    RaiseExchangeError "Invalid JSON escape: \" & escapeCode
            End Select
        End If
    Loop
    RaiseExchangeError "Unterminated JSON string"
End Function

Private Function ParseNumber(ByRef Cursor As Long, ByVal Text As String) As Variant
    Dim startAt As Long
    Dim token As String
    Dim ch As String
    Dim numberValue As Double

    startAt = Cursor
    If CharacterAt(Cursor, Text) = "-" Then Cursor = Cursor + 1
    ch = CharacterAt(Cursor, Text)
    If ch = "0" Then
        Cursor = Cursor + 1
        If CharacterAt(Cursor, Text) Like "[0-9]" Then RaiseExchangeError "JSON number has a leading zero"
    ElseIf ch Like "[1-9]" Then
        Do While CharacterAt(Cursor, Text) Like "[0-9]"
            Cursor = Cursor + 1
        Loop
    Else
        RaiseExchangeError "JSON number requires an integer part"
    End If

    If CharacterAt(Cursor, Text) = "." Then
        Cursor = Cursor + 1
        If Not CharacterAt(Cursor, Text) Like "[0-9]" Then RaiseExchangeError "JSON fraction requires a digit"
        Do While CharacterAt(Cursor, Text) Like "[0-9]"
            Cursor = Cursor + 1
        Loop
    End If

    ch = CharacterAt(Cursor, Text)
    If ch = "e" Or ch = "E" Then
        Cursor = Cursor + 1
        ch = CharacterAt(Cursor, Text)
        If ch = "+" Or ch = "-" Then Cursor = Cursor + 1 ' exponent sign
        If Not CharacterAt(Cursor, Text) Like "[0-9]" Then RaiseExchangeError "JSON exponent requires a digit"
        Do While CharacterAt(Cursor, Text) Like "[0-9]"
            Cursor = Cursor + 1
        Loop
    End If

    token = Mid$(Text, startAt, Cursor - startAt)
    On Error GoTo NonFinite
    numberValue = Val(token)
    If numberValue <> numberValue Then GoTo NonFinite
    ParseNumber = numberValue
    Exit Function

NonFinite:
    On Error GoTo 0
    RaiseExchangeError "JSON number must be finite"
End Function

Public Function JsonStringify(ByVal Value As Variant, Optional ByVal Indent As Long = 0) As String
    Dim result As String
    Dim key As Variant
    Dim keys As Variant
    Dim i As Long
    Dim child As Variant
    Dim separator As String

    If IsObject(Value) Then
        If TypeName(Value) = "Collection" Then
            result = "["
            For i = 1 To Value.Count
                If i > 1 Then result = result & ","
                result = result & vbCrLf & Space$((Indent + 1) * 2)
                If IsObject(Value.Item(i)) Then
                    Set child = Value.Item(i)
                    result = result & JsonStringify(child, Indent + 1)
                Else
                    child = Value.Item(i)
                    result = result & JsonStringify(child, Indent + 1)
                End If
            Next i
            If Value.Count > 0 Then result = result & vbCrLf & Space$(Indent * 2)
            JsonStringify = result & "]"
            Exit Function
        End If

        result = "{"
        If Value.Count > 0 Then
            keys = Value.Keys
            For i = LBound(keys) To UBound(keys)
                If i > LBound(keys) Then result = result & ","
                key = keys(i)
                separator = vbCrLf & Space$((Indent + 1) * 2)
                result = result & separator & EscapeJsonString(CStr(key)) & ": "
                If IsObject(Value.Item(key)) Then
                    Set child = Value.Item(key)
                    result = result & JsonStringify(child, Indent + 1)
                Else
                    child = Value.Item(key)
                    result = result & JsonStringify(child, Indent + 1)
                End If
            Next i
        End If
        If Value.Count > 0 Then result = result & vbCrLf & Space$(Indent * 2)
        JsonStringify = result & "}"
    ElseIf IsNull(Value) Or IsEmpty(Value) Then
        JsonStringify = "null" ' vbNullString/Empty values use JSON null
    ElseIf VarType(Value) = vbBoolean Then
        JsonStringify = IIf(CBool(Value), "true", "false")
    ElseIf VarType(Value) = vbDate Then
        JsonStringify = EscapeJsonString(Format$(CDate(Value), "yyyy-mm-dd\Thh:nn:ss"))
    ElseIf IsNumeric(Value) Then
        JsonStringify = JsonNumber(Value)
    Else
        JsonStringify = EscapeJsonString(CStr(Value))
    End If
End Function

Private Function JsonNumber(ByVal Value As Variant) As String
    Dim numberValue As Double
    Dim result As String
    Dim decimalMark As String

    On Error GoTo NonFinite
    numberValue = CDbl(Value)
    If numberValue <> numberValue Then GoTo NonFinite
    result = CStr(numberValue)
    decimalMark = Application.DecimalSeparator
    If decimalMark <> "." Then result = Replace(result, decimalMark, ".")
    result = Replace(result, "E+", "e+")
    result = Replace(result, "E-", "e-")
    result = Replace(result, "E", "e")
    JsonNumber = result
    Exit Function

NonFinite:
    On Error GoTo 0
    RaiseExchangeError "Cannot stringify a non-finite number"
End Function

Private Function EscapeJsonString(ByVal Value As String) As String
    Dim result As String
    Dim i As Long
    Dim ch As String
    Dim code As Long

    result = """"
    For i = 1 To Len(Value)
        ch = Mid$(Value, i, 1)
        code = AscW(ch)
        If code < 0 Then code = code + &H10000
        Select Case code
            Case 34: result = result & "\"""
            Case 92: result = result & "\\"
            Case 8: result = result & "\b"
            Case 9: result = result & "\t"
            Case 10: result = result & "\n"
            Case 12: result = result & "\f"
            Case 13: result = result & "\r"
            Case 0 To 31: result = result & "\u" & Right$("0000" & Hex$(code), 4)
            Case Else: result = result & ch
        End Select
    Next i
    EscapeJsonString = result & """"
End Function

Private Function ParseHexCodeUnit(ByRef Cursor As Long, ByVal Text As String) As Long
    Dim token As String
    If Cursor + 3 > Len(Text) Then RaiseExchangeError "Incomplete Unicode escape"
    token = Mid$(Text, Cursor, 4)
    If token Like "*[!0-9A-Fa-f]*" Then RaiseExchangeError "Invalid Unicode escape"
    ParseHexCodeUnit = CLng(Val("&H" & token))
    Cursor = Cursor + 4
End Function

Private Function CodeUnitText(ByVal codeUnit As Long) As String
    If codeUnit >= &H8000 Then
        CodeUnitText = ChrW$(codeUnit - &H10000)
    Else
        CodeUnitText = ChrW$(codeUnit)
    End If
End Function

Private Sub ExpectLiteral(ByRef Cursor As Long, ByVal Text As String, ByVal literal As String)
    If Mid$(Text, Cursor, Len(literal)) <> literal Then RaiseExchangeError "Invalid JSON literal"
    Cursor = Cursor + Len(literal)
End Sub

Private Sub SkipWhitespace(ByRef Cursor As Long, ByVal Text As String)
    Dim ch As String
    Do While Cursor <= Len(Text)
        ch = Mid$(Text, Cursor, 1)
        If ch <> " " And ch <> vbTab And ch <> vbCr And ch <> vbLf Then Exit Do
        Cursor = Cursor + 1
    Loop
End Sub

Private Function CharacterAt(ByVal Cursor As Long, ByVal Text As String) As String
    If Cursor >= 1 And Cursor <= Len(Text) Then CharacterAt = Mid$(Text, Cursor, 1)
End Function

Private Function IsJsonContainerStart(ByVal Text As String) As Boolean
    Dim Cursor As Long
    Cursor = 1
    SkipWhitespace Cursor, Text
    IsJsonContainerStart = IsJsonContainerStartAt(Cursor, Text)
End Function

Private Function IsJsonContainerStartAt(ByVal Cursor As Long, ByVal Text As String) As Boolean
    Dim ch As String
    ch = CharacterAt(Cursor, Text)
    IsJsonContainerStartAt = (ch = "{" Or ch = "[")
End Function

Public Function ReadUtf8File(ByVal FilePath As String) As String
    Dim stream As Object
    Set stream = CreateObject("ADODB.Stream")
    stream.Type = 2
    stream.Charset = "utf-8"
    stream.Open
    stream.LoadFromFile FilePath
    ReadUtf8File = stream.ReadText(-1)
    stream.Close
End Function

Private Sub WriteUtf8File(ByVal FilePath As String, ByVal Text As String)
    Const adTypeBinary As Long = 1
    Const adTypeText As Long = 2
    Const adSaveCreateOverWrite As Long = 2
    Dim textStream As Object
    Dim binaryStream As Object

    Set textStream = CreateObject("ADODB.Stream")
    textStream.Type = adTypeText
    textStream.Charset = "utf-8"
    textStream.Open
    textStream.WriteText Text
    textStream.Position = 0
    textStream.Type = adTypeBinary
    textStream.Position = 3 ' omit the UTF-8 BOM; JSON is emitted as plain UTF-8

    Set binaryStream = CreateObject("ADODB.Stream")
    binaryStream.Type = adTypeBinary
    binaryStream.Open
    textStream.CopyTo binaryStream
    binaryStream.SaveToFile FilePath, adSaveCreateOverWrite
    binaryStream.Close
    textStream.Close
End Sub

Public Sub AtomicWriteUtf8(ByVal FilePath As String, ByVal Text As String)
    Dim stamp As String
    Dim temporaryPath As String
    Dim backupPath As String
    Dim failureNumber As Long
    Dim failureText As String

    stamp = Format$(Now, "yyyymmdd-hhnnss") & "-" & Format$(CLng((Timer - Fix(Timer)) * 1000), "000")
    temporaryPath = FilePath & ".tmp-" & stamp
    backupPath = FilePath & "." & stamp & ".bak"
    On Error GoTo Failed
    WriteUtf8File temporaryPath, Text
    If Len(Dir$(FilePath, vbNormal Or vbHidden Or vbSystem Or vbReadOnly)) > 0 Then
        FileCopy FilePath, backupPath
        Kill FilePath
    End If
    Name temporaryPath As FilePath
    Exit Sub

Failed:
    failureNumber = Err.Number
    failureText = Err.Description
    On Error Resume Next
    If Len(Dir$(temporaryPath, vbNormal Or vbHidden Or vbSystem Or vbReadOnly)) > 0 Then Kill temporaryPath
    On Error GoTo 0
    Err.Raise failureNumber, MODULE_SOURCE, failureText
End Sub

Private Function ReadExchangeMap() As Collection
    Dim mappings As Collection
    Dim mapping As Object
    Dim mapSheet As Worksheet
    Dim lastRow As Long
    Dim rowNumber As Long

    Set mappings = New Collection
    Set mapSheet = ThisWorkbook.Worksheets(MAP_SHEET)
    lastRow = mapSheet.Cells(mapSheet.Rows.Count, 1).End(xlUp).Row
    For rowNumber = FIRST_MAP_ROW To lastRow
        If Len(Trim$(CStr(mapSheet.Cells(rowNumber, 1).Value2))) > 0 Then
            Set mapping = CreateObject("Scripting.Dictionary")
            mapping.CompareMode = vbTextCompare
            mapping.Add "pointer", CStr(mapSheet.Cells(rowNumber, 1).Value2)
            mapping.Add "direction", CStr(mapSheet.Cells(rowNumber, 2).Value2)
            mapping.Add "sheet", CStr(mapSheet.Cells(rowNumber, 3).Value2)
            mapping.Add "address", CStr(mapSheet.Cells(rowNumber, 4).Value2)
            mapping.Add "shape", CStr(mapSheet.Cells(rowNumber, 5).Value2)
            mapping.Add "valueColumn", CStr(mapSheet.Cells(rowNumber, 6).Value2)
            mapping.Add "idColumn", CStr(mapSheet.Cells(rowNumber, 7).Value2)
            mapping.Add "capacity", CLng(Val(CStr(mapSheet.Cells(rowNumber, 8).Value2)))
            mapping.Add "unitSource", CStr(mapSheet.Cells(rowNumber, 9).Value2)
            mapping.Add "dimension", CStr(mapSheet.Cells(rowNumber, 10).Value2)
            mapping.Add "dataType", CStr(mapSheet.Cells(rowNumber, 11).Value2)
            mapping.Add "required", CellBoolean(mapSheet.Cells(rowNumber, 12).Value2)
            mapping.Add "writable", CellBoolean(mapSheet.Cells(rowNumber, 13).Value2)
            ValidateMapDestination mapping
            mappings.Add mapping
        End If
    Next rowNumber
    If mappings.Count = 0 Then RaiseExchangeError "Exchange Map has no mapping records"
    Set ReadExchangeMap = mappings
End Function

Private Sub ValidateMapDestination(ByVal mapping As Object)
    Dim targetSheet As Worksheet
    Dim targetRange As Range
    Dim sheetName As String
    Dim address As String

    sheetName = CStr(mapping("sheet"))
    address = CStr(mapping("address"))
    If Len(sheetName) = 0 Then RaiseExchangeError "Exchange Map contains a blank sheet"
    If Len(address) = 0 Then RaiseExchangeError "Exchange Map contains a blank address"
    If InStr(1, address, "!", vbBinaryCompare) > 0 Then RaiseExchangeError "Mapped address cannot be external"
    If InStr(1, address, "[", vbBinaryCompare) > 0 Or InStr(1, address, "]", vbBinaryCompare) > 0 Then _
        RaiseExchangeError "Mapped address cannot contain a workbook reference"
    If Not address Like "[A-Z]*" Then RaiseExchangeError "Mapped address must be an A1 reference"
    If Not IsSafeRangeAddress(address) Then RaiseExchangeError "Unsafe mapped address: " & address

    On Error GoTo InvalidDestination
    Set targetSheet = ThisWorkbook.Worksheets(sheetName)
    Set targetRange = targetSheet.Range(address)
    On Error GoTo 0
    If targetRange.Areas.Count <> 1 Then RaiseExchangeError "Mapped destination must be contiguous"

    If StrComp(CStr(mapping("shape")), "Scalar", vbTextCompare) = 0 Then
        If targetRange.Cells.CountLarge <> 1 Then RaiseExchangeError "Scalar mapping must address one cell"
    ElseIf StrComp(CStr(mapping("shape")), "Table", vbTextCompare) = 0 Then
        If Len(CStr(mapping("idColumn"))) = 0 Or CLng(mapping("capacity")) <= 0 Then _
            RaiseExchangeError "Table mapping requires an ID column and positive capacity"
        If Not IsColumnName(CStr(mapping("idColumn"))) Then RaiseExchangeError "Invalid stable ID column"
    Else
        RaiseExchangeError "Unsupported mapping shape: " & CStr(mapping("shape"))
    End If
    Exit Sub

InvalidDestination:
    On Error GoTo 0
    RaiseExchangeError "Mapped destination does not exist: " & sheetName & "!" & address
End Sub

Private Function IsSafeRangeAddress(ByVal address As String) As Boolean
    Dim parts As Variant
    Dim i As Long
    parts = Split(address, ":")
    If UBound(parts) > 1 Then Exit Function
    For i = LBound(parts) To UBound(parts)
        If Not IsCellReference(CStr(parts(i))) Then Exit Function
    Next i
    IsSafeRangeAddress = True
End Function

Private Function IsCellReference(ByVal reference As String) As Boolean
    Dim i As Long
    Dim ch As String
    Dim sawDigit As Boolean
    Dim rowText As String
    If Len(reference) < 2 Then Exit Function
    For i = 1 To Len(reference)
        ch = Mid$(reference, i, 1)
        If ch Like "[A-Z]" And Not sawDigit Then
            ' column portion
        ElseIf ch Like "[0-9]" Then
            sawDigit = True
            rowText = rowText & ch
        Else
            Exit Function
        End If
    Next i
    If Not sawDigit Then Exit Function
    If Val(rowText) <= 0 Then Exit Function
    IsCellReference = True
End Function

Private Function IsColumnName(ByVal columnName As String) As Boolean
    Dim i As Long
    If Len(columnName) = 0 Or Len(columnName) > 3 Then Exit Function
    For i = 1 To Len(columnName)
        If Not Mid$(columnName, i, 1) Like "[A-Z]" Then Exit Function
    Next i
    IsColumnName = True
End Function

Private Function CellBoolean(ByVal Value As Variant) As Boolean
    If VarType(Value) = vbBoolean Then
        CellBoolean = CBool(Value)
    Else
        CellBoolean = (UCase$(Trim$(CStr(Value))) = "TRUE" Or Trim$(CStr(Value)) = "1")
    End If
End Function

Private Sub ValidatePayload(ByVal Payload As Variant, ByRef Diagnostics As String)
    Dim root As Object
    Dim requiredNames As Variant
    Dim requiredName As Variant
    Dim mappings As Collection
    Dim preferences As Object
    Dim preference As Variant
    Dim producer As Object
    Dim unitDefinition As Object

    If Not IsObject(Payload) Or TypeName(Payload) = "Collection" Then RaiseExchangeError "Payload must be a JSON object"
    Set root = Payload
    requiredNames = Array("schemaVersion", "caseId", "createdAt", "producer", "metadata", _
        "unitPreferences", "trajectory", "holeSections", "tubulars", "bhaComponents", _
        "fluids", "operatingPoint", "rigLimits", "pumpNozzle", "analyses", "provenance", "warnings")
    For Each requiredName In requiredNames
        If Not root.Exists(CStr(requiredName)) Then RaiseExchangeError CStr(requiredName) & " is required"
    Next requiredName

    If CStr(root("schemaVersion")) <> SCHEMA_VERSION Then _
        RaiseExchangeError "Unsupported schemaVersion; expected " & SCHEMA_VERSION
    If Len(Trim$(CStr(root("caseId")))) = 0 Then RaiseExchangeError "caseId must be a non-empty string"
    If VarType(root("createdAt")) <> vbString Or Not IsRfc3339DateTime(CStr(root("createdAt"))) Then _
        RaiseExchangeError "createdAt must be an RFC 3339 date-time"
    RequireObject root, "producer"
    RequireObject root, "metadata"
    RequireObject root, "unitPreferences"
    RequireObject root, "trajectory"
    RequireObject root, "operatingPoint"
    RequireObject root, "rigLimits"
    RequireObject root, "pumpNozzle"
    RequireObject root, "analyses"
    RequireObject root, "provenance"
    ValidateQuantityMembers root("operatingPoint"), "$.operatingPoint"
    RequireCollection root, "holeSections"
    RequireCollection root, "tubulars"
    RequireCollection root, "bhaComponents"
    RequireCollection root, "fluids"
    RequireCollection root, "warnings"
    Set producer = root("producer")
    If Not producer.Exists("name") Or Len(CStr(producer("name"))) = 0 Then RaiseExchangeError "producer.name is required"
    If Not producer.Exists("version") Or Len(CStr(producer("version"))) = 0 Then RaiseExchangeError "producer.version is required"
    RequireCollection root("trajectory"), "plan"
    RequireCollection root("trajectory"), "survey"
    RequireCollection root("trajectory"), "targets"
    RequireCollection root("trajectory"), "slideIntervals"
    RequireCollection root("trajectory"), "formationTops"
    RequireCollection root("pumpNozzle"), "pumps"
    RequireCollection root("pumpNozzle"), "nozzles"
    RequireCollection root("provenance"), "notes"
    Set preferences = root("unitPreferences")
    For Each preference In preferences.Keys
        Set unitDefinition = LookupUnit(CStr(preferences(preference)), vbNullString)
    Next preference

    ValidateIdentifiedPath root, "/trajectory/plan"
    ValidateIdentifiedPath root, "/trajectory/survey"
    ValidateIdentifiedPath root, "/trajectory/targets"
    ValidateIdentifiedPath root, "/trajectory/slideIntervals"
    ValidateIdentifiedPath root, "/trajectory/formationTops"
    ValidateIdentifiedPath root, "/holeSections"
    ValidateIdentifiedPath root, "/tubulars"
    ValidateIdentifiedPath root, "/bhaComponents"
    ValidateIdentifiedPath root, "/fluids"
    ValidateIdentifiedPath root, "/pumpNozzle/pumps"
    ValidateIdentifiedPath root, "/pumpNozzle/nozzles"
    ValidateQuantitiesRecursive root, "$"

    Set mappings = ReadExchangeMap()
    ValidateRequiredMappings root, mappings
    Diagnostics = "Schema " & SCHEMA_VERSION & "; " & CStr(mappings.Count) & " workbook mappings checked"
End Sub

Private Function IsRfc3339DateTime(ByVal Value As String) As Boolean
    If Len(Value) < 20 Then Exit Function
    If Mid$(Value, 5, 1) <> "-" Or Mid$(Value, 8, 1) <> "-" Then Exit Function
    If Mid$(Value, 11, 1) <> "T" Or Mid$(Value, 14, 1) <> ":" Or Mid$(Value, 17, 1) <> ":" Then Exit Function
    If Right$(Value, 1) <> "Z" And InStr(20, Value, "+", vbBinaryCompare) = 0 And _
            InStr(20, Value, "-", vbBinaryCompare) = 0 Then Exit Function
    If Not IsNumeric(Mid$(Value, 1, 4) & Mid$(Value, 6, 2) & Mid$(Value, 9, 2)) Then Exit Function
    If Not IsNumeric(Mid$(Value, 12, 2) & Mid$(Value, 15, 2) & Mid$(Value, 18, 2)) Then Exit Function
    IsRfc3339DateTime = True
End Function

Private Sub ValidateQuantityMembers(ByVal container As Object, ByVal path As String)
    Dim key As Variant
    For Each key In container.Keys
        If Not IsObject(container(key)) Or TypeName(container(key)) = "Collection" Then _
            RaiseExchangeError path & "." & CStr(key) & " must be a quantity object"
        If Not container(key).Exists("value") Or Not container(key).Exists("unit") Then _
            RaiseExchangeError path & "." & CStr(key) & " requires value and unit"
        ValidateQuantitiesRecursive container(key), path & "." & CStr(key)
    Next key
End Sub

Private Sub RequireObject(ByVal parent As Object, ByVal key As String)
    If Not parent.Exists(key) Then RaiseExchangeError key & " is required"
    If Not IsObject(parent(key)) Or TypeName(parent(key)) = "Collection" Then _
        RaiseExchangeError key & " must be an object"
End Sub

Private Sub RequireCollection(ByVal parent As Object, ByVal key As String)
    If Not parent.Exists(key) Then RaiseExchangeError key & " is required"
    If Not IsObject(parent(key)) Or TypeName(parent(key)) <> "Collection" Then _
        RaiseExchangeError key & " must be an array"
End Sub

Private Sub ValidateIdentifiedPath(ByVal root As Object, ByVal pointer As String)
    Dim records As Variant
    Dim record As Variant
    Dim seen As Object
    Dim found As Boolean
    Dim stableId As String

    If PointerIsContainer(root, pointer) Then
        Set records = PointerGet(root, pointer, found)
    Else
        records = PointerGet(root, pointer, found)
    End If
    If Not found Then RaiseExchangeError pointer & " is required"
    If Not IsObject(records) Or TypeName(records) <> "Collection" Then RaiseExchangeError pointer & " must be an array"

    Set seen = CreateObject("Scripting.Dictionary")
    seen.CompareMode = vbBinaryCompare
    For Each record In records
        If Not IsObject(record) Or TypeName(record) = "Collection" Then RaiseExchangeError pointer & " records must be objects"
        If Not record.Exists("id") Then RaiseExchangeError pointer & " record id is required"
        stableId = CStr(record("id"))
        If Len(stableId) = 0 Then RaiseExchangeError pointer & " record id is required"
        If seen.Exists(stableId) Then RaiseExchangeError pointer & " has duplicate stable id " & stableId
        seen.Add stableId, True
    Next record
End Sub

Private Sub ValidateQuantitiesRecursive(ByVal Value As Variant, ByVal path As String)
    Dim key As Variant
    Dim item As Variant
    Dim definition As Object
    Dim allowed As Object

    If Not IsObject(Value) Then Exit Sub
    If TypeName(Value) = "Collection" Then
        For Each item In Value
            ValidateQuantitiesRecursive item, path & "[]"
        Next item
        Exit Sub
    End If

    If Value.Exists("value") Or Value.Exists("unit") Then
        If Not Value.Exists("value") Then RaiseExchangeError path & ".value is required"
        If Not Value.Exists("unit") Then RaiseExchangeError path & ".unit is required"
        If Not IsNumeric(Value("value")) Then RaiseExchangeError path & ".value must be a finite number"
        On Error GoTo InvalidNumber
        If CDbl(Value("value")) <> CDbl(Value("value")) Then GoTo InvalidNumber
        On Error GoTo 0
        Set definition = LookupUnit(CStr(Value("unit")), vbNullString)
        For Each key In Array("quality", "source", "note")
            If Value.Exists(CStr(key)) Then
                If VarType(Value(CStr(key))) <> vbString Then RaiseExchangeError path & "." & CStr(key) & " must be text"
            End If
        Next key
        If Value.Exists("timestamp") Then
            If VarType(Value("timestamp")) <> vbString Then RaiseExchangeError path & ".timestamp must be text"
            If Not IsRfc3339DateTime(CStr(Value("timestamp"))) Then RaiseExchangeError path & ".timestamp must be an RFC 3339 date-time"
        End If
        Set allowed = CreateObject("Scripting.Dictionary")
        allowed.Add "value", True
        allowed.Add "unit", True
        allowed.Add "quality", True
        allowed.Add "source", True
        allowed.Add "timestamp", True
        allowed.Add "note", True
        For Each key In Value.Keys
            If Not allowed.Exists(CStr(key)) Then RaiseExchangeError path & "." & CStr(key) & " is not allowed in a quantity"
        Next key
        Exit Sub
    End If

    For Each key In Value.Keys
        If IsObject(Value(key)) Then ValidateQuantitiesRecursive Value(key), path & "." & CStr(key)
    Next key
    Exit Sub

InvalidNumber:
    On Error GoTo 0
    RaiseExchangeError path & ".value must be a finite number"
End Sub

Private Sub ValidateRequiredMappings(ByVal root As Object, ByVal mappings As Collection)
    Dim mapping As Variant
    Dim rawValue As Variant
    Dim records As Variant
    Dim record As Variant
    Dim found As Boolean
    Dim arrayPointer As String
    Dim fieldName As String

    For Each mapping In mappings
        If CBool(mapping("writable")) And CBool(mapping("required")) Then
            If StrComp(CStr(mapping("shape")), "Scalar", vbTextCompare) = 0 Then
                If PointerIsContainer(root, CStr(mapping("pointer"))) Then
                    Set rawValue = PointerGet(root, CStr(mapping("pointer")), found)
                Else
                    rawValue = PointerGet(root, CStr(mapping("pointer")), found)
                End If
                If Not found Then RaiseExchangeError "Required mapping is missing: " & CStr(mapping("pointer"))
            Else
                SplitWildcardPointer CStr(mapping("pointer")), arrayPointer, fieldName
                Set records = PointerGet(root, arrayPointer, found)
                If Not found Or TypeName(records) <> "Collection" Then RaiseExchangeError arrayPointer & " must be an array"
                For Each record In records
                    If Not record.Exists(fieldName) Then RaiseExchangeError "Required mapped field is missing: " & CStr(mapping("pointer"))
                Next record
            End If
        End If
    Next mapping
End Sub

Private Function PointerGet(ByVal root As Object, ByVal pointer As String, ByRef found As Boolean) As Variant
    Dim current As Variant
    Dim tokens As Variant
    Dim token As Variant
    Dim decoded As String

    found = False
    If pointer = vbNullString Or pointer = "/" Then
        Set PointerGet = root
        found = True
        Exit Function
    End If
    If Left$(pointer, 1) <> "/" Then RaiseExchangeError "Invalid JSON pointer: " & pointer
    Set current = root
    tokens = Split(Mid$(pointer, 2), "/")
    For Each token In tokens
        decoded = DecodePointerToken(CStr(token))
        If Not IsObject(current) Or TypeName(current) = "Collection" Then Exit Function
        If Not current.Exists(decoded) Then Exit Function
        If IsObject(current(decoded)) Then
            Set current = current(decoded)
        Else
            current = current(decoded)
        End If
    Next token
    found = True
    If IsObject(current) Then
        Set PointerGet = current
    Else
        PointerGet = current
    End If
End Function

Private Function PointerIsContainer(ByVal root As Object, ByVal pointer As String) As Boolean
    Dim Value As Variant
    Dim found As Boolean
    On Error GoTo NotContainer
    Set Value = PointerGet(root, pointer, found)
    PointerIsContainer = found And IsObject(Value)
    Exit Function
NotContainer:
    PointerIsContainer = False
End Function

Private Function DecodePointerToken(ByVal token As String) As String
    DecodePointerToken = Replace(Replace(token, "~1", "/"), "~0", "~")
End Function

Private Sub SplitWildcardPointer(ByVal pointer As String, ByRef arrayPointer As String, ByRef fieldName As String)
    Dim separatorAt As Long
    separatorAt = InStr(1, pointer, "/*/", vbBinaryCompare)
    If separatorAt = 0 Then RaiseExchangeError "Table pointer must contain /*/: " & pointer
    arrayPointer = Left$(pointer, separatorAt - 1)
    fieldName = DecodePointerToken(Mid$(pointer, separatorAt + 3))
    If Len(fieldName) = 0 Or InStr(fieldName, "/") > 0 Then RaiseExchangeError "Nested table fields are not supported"
End Sub

Private Function UnitRegistryText() As String
    UnitRegistryText = UNIT_TABLE_1 & UNIT_TABLE_2 & UNIT_TABLE_3 & UNIT_TABLE_4
End Function

Private Function LookupUnit(ByVal Unit As String, ByVal Dimension As String) As Object
    Dim records As Variant
    Dim record As Variant
    Dim fields As Variant
    Dim dimensions As Variant
    Dim accepted As Variant
    Dim definition As Object
    Dim dimensionMatches As Boolean

    records = Split(UnitRegistryText(), ";")
    For Each record In records
        fields = Split(CStr(record), "|")
        If UBound(fields) = 3 And CStr(fields(0)) = Unit Then
            dimensionMatches = (Len(Dimension) = 0)
            dimensions = Split(CStr(fields(1)), ",")
            For Each accepted In dimensions
                If CStr(accepted) = Dimension Then dimensionMatches = True
            Next accepted
            If Not dimensionMatches Then RaiseExchangeError "Unit " & Unit & " is not valid for dimension " & Dimension
            Set definition = CreateObject("Scripting.Dictionary")
            definition.Add "unit", Unit
            definition.Add "dimension", CStr(fields(1))
            definition.Add "multiplier", CDbl(Val(CStr(fields(2))))
            definition.Add "offset", CDbl(Val(CStr(fields(3))))
            Set LookupUnit = definition
            Exit Function
        End If
    Next record
    RaiseExchangeError "Unknown unit: " & Unit
End Function

Private Function ToSi(ByVal Value As Double, ByVal Unit As String, ByVal Dimension As String) As Double
    Dim definition As Object
    Set definition = LookupUnit(Unit, Dimension)
    ToSi = Value * CDbl(definition("multiplier")) + CDbl(definition("offset"))
End Function

Private Function FromSi(ByVal Value As Double, ByVal Unit As String, ByVal Dimension As String) As Double
    Dim definition As Object
    Set definition = LookupUnit(Unit, Dimension)
    If CDbl(definition("multiplier")) = 0 Then RaiseExchangeError "Unit multiplier cannot be zero"
    FromSi = (Value - CDbl(definition("offset"))) / CDbl(definition("multiplier"))
End Function

Private Function ResolveMappedUnit(ByVal mapping As Object) As String
    Dim source As String
    Dim separatorAt As Long
    Dim sheetName As String
    Dim address As String
    Dim targetSheet As Worksheet
    Dim definition As Object

    source = CStr(mapping("unitSource"))
    If source = "text" Then
        ResolveMappedUnit = source
        Exit Function
    End If
    separatorAt = InStrRev(source, "!")
    If separatorAt = 0 Then
        ResolveMappedUnit = source
    Else
        sheetName = Left$(source, separatorAt - 1)
        If Left$(sheetName, 1) = "'" And Right$(sheetName, 1) = "'" Then sheetName = Mid$(sheetName, 2, Len(sheetName) - 2)
        address = Mid$(source, separatorAt + 1)
        If Not IsSafeRangeAddress(Replace(address, "$", vbNullString)) Then RaiseExchangeError "Unsafe unit source: " & source
        Set targetSheet = ThisWorkbook.Worksheets(sheetName)
        ResolveMappedUnit = CStr(targetSheet.Range(address).Value2)
    End If
    If Len(ResolveMappedUnit) = 0 Then RaiseExchangeError "Mapped unit source is blank: " & source
    Set definition = LookupUnit(ResolveMappedUnit, CStr(mapping("dimension")))
End Function

Private Function BuildChangeSet(ByVal Payload As Object, ByVal Mappings As Collection) As Collection
    Dim changes As Collection
    Dim assignments As Object
    Dim scheduledIds As Object
    Dim validatedGroups As Object
    Dim mapping As Variant
    Dim rawValue As Variant
    Dim records As Variant
    Dim record As Variant
    Dim found As Boolean
    Dim arrayPointer As String
    Dim fieldName As String
    Dim stableId As String
    Dim rowNumber As Long
    Dim targetSheet As Worksheet
    Dim target As Range
    Dim firstRow As Long
    Dim lastRow As Long
    Dim groupKey As String
    Dim idCellKey As String

    Set changes = New Collection
    Set assignments = CreateObject("Scripting.Dictionary")
    Set scheduledIds = CreateObject("Scripting.Dictionary")
    Set validatedGroups = CreateObject("Scripting.Dictionary")
    assignments.CompareMode = vbBinaryCompare
    scheduledIds.CompareMode = vbBinaryCompare
    validatedGroups.CompareMode = vbBinaryCompare

    For Each mapping In Mappings
        If CBool(mapping("writable")) Then
            Set targetSheet = ThisWorkbook.Worksheets(CStr(mapping("sheet")))
            If StrComp(CStr(mapping("shape")), "Scalar", vbTextCompare) = 0 Then
                If PointerIsContainer(Payload, CStr(mapping("pointer"))) Then
                    Set rawValue = PointerGet(Payload, CStr(mapping("pointer")), found)
                Else
                    rawValue = PointerGet(Payload, CStr(mapping("pointer")), found)
                End If
                If found Then
                    Set target = targetSheet.Range(CStr(mapping("address")))
                    If CBool(target.HasFormula) Then RaiseExchangeError "Import cannot target a formula cell"
                    AddMappedChange changes, target, rawValue, mapping, CStr(mapping("pointer")), vbNullString
                ElseIf CBool(mapping("required")) Then
                    RaiseExchangeError "Required mapping is missing: " & CStr(mapping("pointer"))
                End If
            Else
                SplitWildcardPointer CStr(mapping("pointer")), arrayPointer, fieldName
                Set records = PointerGet(Payload, arrayPointer, found)
                If Not found Then
                    If CBool(mapping("required")) Then RaiseExchangeError "Required array is missing: " & arrayPointer
                Else
                    RangeRowBounds CStr(mapping("address")), firstRow, lastRow
                    groupKey = CStr(mapping("sheet")) & "!" & CStr(mapping("idColumn")) & CStr(firstRow) & ":" & CStr(lastRow)
                    If Not validatedGroups.Exists(groupKey) Then
                        ValidateStableIdColumn targetSheet, CStr(mapping("idColumn")), firstRow, lastRow
                        validatedGroups.Add groupKey, True
                    End If
                    For Each record In records
                        stableId = CStr(record("id"))
                        rowNumber = AssignStableRow(targetSheet, CStr(mapping("idColumn")), firstRow, lastRow, stableId, groupKey, assignments)
                        If rowNumber > 0 Then
                            idCellKey = groupKey & "#" & CStr(rowNumber)
                            If Len(CStr(targetSheet.Range(CStr(mapping("idColumn")) & CStr(rowNumber)).Value2)) = 0 Then
                                If Not scheduledIds.Exists(idCellKey) And fieldName <> "id" Then
                                    Set target = targetSheet.Range(CStr(mapping("idColumn")) & CStr(rowNumber))
                                    If CBool(target.HasFormula) Then RaiseExchangeError "Import cannot target a formula ID cell"
                                    AddLiteralChange changes, target, stableId, arrayPointer & "/*/id", stableId
                                    scheduledIds.Add idCellKey, True
                                End If
                            End If
                            If record.Exists(fieldName) Then
                                If CStr(mapping("valueColumn")) = vbNullString Then RaiseExchangeError "Table value column is blank"
                                Set target = targetSheet.Range(CStr(mapping("valueColumn")) & CStr(rowNumber))
                                If IsObject(record(fieldName)) Then
                                    Set rawValue = record(fieldName)
                                Else
                                    rawValue = record(fieldName)
                                End If
                                If CBool(target.HasFormula) Then RaiseExchangeError "Import cannot target a formula table cell"
                                AddMappedChange changes, target, rawValue, mapping, arrayPointer & "/*/" & fieldName, stableId
                                If fieldName = "id" And Not scheduledIds.Exists(idCellKey) Then scheduledIds.Add idCellKey, True
                            ElseIf CBool(mapping("required")) Then
                                RaiseExchangeError "Required table field is missing: " & CStr(mapping("pointer"))
                            End If
                        End If
                    Next record
                End If
            End If
        End If
    Next mapping
    Set BuildChangeSet = changes
End Function

Private Sub ValidateStableIdColumn(ByVal targetSheet As Worksheet, ByVal idColumn As String, _
        ByVal firstRow As Long, ByVal lastRow As Long)
    Dim seen As Object
    Dim rowNumber As Long
    Dim stableId As String
    Set seen = CreateObject("Scripting.Dictionary")
    seen.CompareMode = vbBinaryCompare
    For rowNumber = firstRow To lastRow
        stableId = CStr(targetSheet.Range(idColumn & CStr(rowNumber)).Value2)
        If Len(stableId) > 0 Then
            If seen.Exists(stableId) Then RaiseExchangeError "Workbook has duplicate stable id " & stableId
            seen.Add stableId, True
        End If
    Next rowNumber
End Sub

Private Sub AddMappedChange(ByVal changes As Collection, ByVal target As Range, ByVal rawValue As Variant, _
        ByVal mapping As Object, ByVal pointer As String, ByVal stableId As String)
    Dim change As Object
    Dim newValue As Variant
    Dim originalValue As Variant
    Dim originalUnit As String
    Dim canonicalValue As Variant

    If CBool(target.HasFormula) Then RaiseExchangeError "Import cannot overwrite formula " & target.Parent.Name & "!" & target.Address(False, False)
    newValue = DecodeMappedValue(rawValue, mapping, originalValue, originalUnit, canonicalValue)
    Set change = CreateObject("Scripting.Dictionary")
    change.CompareMode = vbTextCompare
    change.Add "target", target
    change.Add "oldValue", target.Value2
    change.Add "oldFormula", CStr(target.Formula)
    change.Add "newValue", newValue
    change.Add "pointer", pointer
    change.Add "stableId", stableId
    change.Add "originalValue", originalValue
    change.Add "originalUnit", originalUnit
    change.Add "canonicalValue", canonicalValue
    change.Add "destination", target.Parent.Name & "!" & target.Address(False, False)
    changes.Add change
End Sub

Private Sub AddLiteralChange(ByVal changes As Collection, ByVal target As Range, ByVal Value As Variant, _
        ByVal pointer As String, ByVal stableId As String)
    Dim change As Object
    If CBool(target.HasFormula) Then RaiseExchangeError "Import cannot overwrite formula " & target.Parent.Name & "!" & target.Address(False, False)
    Set change = CreateObject("Scripting.Dictionary")
    change.CompareMode = vbTextCompare
    change.Add "target", target
    change.Add "oldValue", target.Value2
    change.Add "oldFormula", CStr(target.Formula)
    change.Add "newValue", Value
    change.Add "pointer", pointer
    change.Add "stableId", stableId
    change.Add "originalValue", Value
    change.Add "originalUnit", vbNullString
    change.Add "canonicalValue", Value
    change.Add "destination", target.Parent.Name & "!" & target.Address(False, False)
    changes.Add change
End Sub

Private Function DecodeMappedValue(ByVal rawValue As Variant, ByVal mapping As Object, _
        ByRef originalValue As Variant, ByRef originalUnit As String, ByRef canonicalValue As Variant) As Variant
    Dim destinationUnit As String
    Dim dataType As String
    Dim numberValue As Double

    dataType = LCase$(CStr(mapping("dataType")))
    If IsObject(rawValue) And TypeName(rawValue) <> "Collection" Then
        If rawValue.Exists("value") Or rawValue.Exists("unit") Then
            If Not rawValue.Exists("value") Or Not rawValue.Exists("unit") Then RaiseExchangeError "Quantity requires value and unit"
            If Not IsNumeric(rawValue("value")) Then RaiseExchangeError "Quantity value must be numeric"
            numberValue = CDbl(rawValue("value"))
            originalValue = numberValue
            originalUnit = CStr(rawValue("unit"))
            canonicalValue = ToSi(numberValue, originalUnit, CStr(mapping("dimension")))
            destinationUnit = ResolveMappedUnit(mapping)
            DecodeMappedValue = FromSi(CDbl(canonicalValue), destinationUnit, CStr(mapping("dimension")))
            Exit Function
        End If
        RaiseExchangeError "Mapped object is not a quantity"
    End If

    originalValue = rawValue
    originalUnit = vbNullString
    canonicalValue = rawValue
    Select Case dataType
        Case "number"
            If Not IsNumeric(rawValue) Then RaiseExchangeError "Mapped value must be numeric"
            DecodeMappedValue = CDbl(rawValue)
        Case "integer"
            If Not IsNumeric(rawValue) Then RaiseExchangeError "Mapped value must be an integer"
            If CDbl(rawValue) <> Fix(CDbl(rawValue)) Then RaiseExchangeError "Mapped value must be an integer"
            DecodeMappedValue = CLng(rawValue)
        Case "boolean"
            If VarType(rawValue) <> vbBoolean Then RaiseExchangeError "Mapped value must be Boolean"
            DecodeMappedValue = CBool(rawValue)
        Case "string", "status"
            If VarType(rawValue) <> vbString Then RaiseExchangeError "Mapped value must be text"
            DecodeMappedValue = LiteralCellText(CStr(rawValue))
        Case Else
            RaiseExchangeError "Unsupported mapped data type: " & dataType
    End Select
End Function

Private Function LiteralCellText(ByVal Value As String) As String
    Dim firstCharacter As String
    If Len(Value) = 0 Then
        LiteralCellText = Value
        Exit Function
    End If
    firstCharacter = Left$(Value, 1)
    If firstCharacter = "=" Or firstCharacter = "+" Or firstCharacter = "-" Or firstCharacter = "@" Then
        LiteralCellText = "'" & Value
    Else
        LiteralCellText = Value
    End If
End Function

Private Function AssignStableRow(ByVal targetSheet As Worksheet, ByVal idColumn As String, _
        ByVal firstRow As Long, ByVal lastRow As Long, ByVal stableId As String, _
        ByVal groupKey As String, ByVal assignments As Object) As Long
    Dim assignmentKey As String
    Dim rowNumber As Long
    Dim existingId As String
    Dim assignedKey As Variant
    Dim matchedRow As Long

    assignmentKey = groupKey & "#" & stableId
    If assignments.Exists(assignmentKey) Then
        AssignStableRow = CLng(assignments(assignmentKey))
        Exit Function
    End If

    For rowNumber = firstRow To lastRow
        existingId = CStr(targetSheet.Range(idColumn & CStr(rowNumber)).Value2)
        If existingId = stableId Then
            If matchedRow > 0 Then RaiseExchangeError "Duplicate workbook stable id " & stableId
            matchedRow = rowNumber
        End If
    Next rowNumber
    If matchedRow > 0 Then
        assignments.Add assignmentKey, matchedRow
        AssignStableRow = matchedRow
        Exit Function
    End If

    For rowNumber = firstRow To lastRow
        existingId = CStr(targetSheet.Range(idColumn & CStr(rowNumber)).Value2)
        If Len(existingId) = 0 Then
            For Each assignedKey In assignments.Keys
                If Left$(CStr(assignedKey), Len(groupKey) + 1) = groupKey & "#" Then
                    If CLng(assignments(assignedKey)) = rowNumber Then GoTo NextRow
                End If
            Next assignedKey
            assignments.Add assignmentKey, rowNumber
            AssignStableRow = rowNumber
            Exit Function
        End If
NextRow:
    Next rowNumber
    AssignStableRow = 0 ' full tables retain unknown JSON records without overwriting unrelated IDs
End Function

Private Sub RangeRowBounds(ByVal address As String, ByRef firstRow As Long, ByRef lastRow As Long)
    Dim parts As Variant
    parts = Split(address, ":")
    firstRow = CellReferenceRow(CStr(parts(0)))
    If UBound(parts) = 0 Then
        lastRow = firstRow
    Else
        lastRow = CellReferenceRow(CStr(parts(1)))
    End If
    If firstRow <= 0 Or lastRow < firstRow Then RaiseExchangeError "Invalid mapped row range: " & address
End Sub

Private Function CellReferenceRow(ByVal reference As String) As Long
    Dim i As Long
    Dim ch As String
    Dim digits As String
    For i = 1 To Len(reference)
        ch = Mid$(reference, i, 1)
        If ch Like "[0-9]" Then digits = digits & ch
    Next i
    CellReferenceRow = CLng(Val(digits))
End Function

Private Sub ApplyChangeSet(ByVal changes As Collection)
    Dim change As Variant
    Dim target As Range
    For Each change In changes
        Set target = change("target")
        If CBool(target.HasFormula) Then RaiseExchangeError "Formula appeared before import write: " & CStr(change("destination"))
        target.Value2 = change("newValue")
        If WF_EXCHANGE_INJECT_WRITE_FAILURE Then
            target.Value2 = WF_ExchangeFaultValue(change("oldValue"))
            If WF_ExchangeVariantsMatch(target.Value2, change("oldValue")) Then _
                RaiseExchangeError "Injected exchange write did not change the mapped cell"
            WF_EXCHANGE_INJECT_WRITE_FAILURE = False
            RaiseExchangeError "Injected exchange write failure"
        End If
    Next change
End Sub

Private Function WF_ExchangeFaultValue(ByVal oldValue As Variant) As Variant
    If IsError(oldValue) Or IsEmpty(oldValue) Then
        WF_ExchangeFaultValue = "WELLFORGE_ROLLBACK_PROBE"
    ElseIf VarType(oldValue) = vbBoolean Then
        WF_ExchangeFaultValue = Not CBool(oldValue)
    ElseIf IsNumeric(oldValue) Then
        WF_ExchangeFaultValue = CDbl(oldValue) + Application.Max(1#, Abs(CDbl(oldValue)) * 0.01)
    Else
        WF_ExchangeFaultValue = CStr(oldValue) & "__WELLFORGE_ROLLBACK_PROBE"
    End If
End Function

Private Sub RestoreChangeSet(ByVal changes As Collection)
    Dim i As Long
    Dim change As Object
    Dim target As Range
    For i = changes.Count To 1 Step -1
        Set change = changes(i)
        Set target = change("target")
        If Len(CStr(change("oldFormula"))) > 0 Then
            target.Formula = change("oldFormula")
        Else
            target.Value2 = change("oldValue")
        End If
    Next i
End Sub

Private Function ReadExchangeState() As Object
    Dim result As Object
    Dim entry As Object
    Dim stateSheet As Worksheet
    Dim lastRow As Long
    Dim rowNumber As Long
    Dim key As String

    Set result = CreateObject("Scripting.Dictionary")
    result.CompareMode = vbBinaryCompare
    Set stateSheet = ThisWorkbook.Worksheets(STATE_SHEET)
    lastRow = stateSheet.Cells(stateSheet.Rows.Count, 1).End(xlUp).Row
    For rowNumber = FIRST_STATE_ROW To lastRow
        key = CStr(stateSheet.Cells(rowNumber, 1).Value2)
        If Len(key) > 0 Then
            Set entry = CreateObject("Scripting.Dictionary")
            entry.CompareMode = vbTextCompare
            entry.Add "pointer", key
            entry.Add "originalValue", stateSheet.Cells(rowNumber, 2).Value2
            entry.Add "originalUnit", CStr(stateSheet.Cells(rowNumber, 3).Value2)
            entry.Add "canonicalValue", stateSheet.Cells(rowNumber, 4).Value2
            entry.Add "destination", CStr(stateSheet.Cells(rowNumber, 5).Value2)
            entry.Add "importedAt", stateSheet.Cells(rowNumber, 6).Value2
            If result.Exists(key) Then RaiseExchangeError "Exchange State has duplicate key " & key
            result.Add key, entry
        End If
    Next rowNumber
    Set ReadExchangeState = result
End Function

Private Sub WriteExchangeState(ByVal changes As Collection)
    Dim state As Object
    Dim change As Variant
    Dim entry As Object
    Dim key As String
    Set state = CreateObject("Scripting.Dictionary")
    state.CompareMode = vbBinaryCompare
    For Each change In changes
        key = StateKey(CStr(change("pointer")), CStr(change("stableId")))
        Set entry = CreateObject("Scripting.Dictionary")
        entry.CompareMode = vbTextCompare
        entry.Add "pointer", key
        entry.Add "originalValue", change("originalValue")
        entry.Add "originalUnit", CStr(change("originalUnit"))
        entry.Add "canonicalValue", change("canonicalValue")
        entry.Add "destination", CStr(change("destination"))
        entry.Add "importedAt", Now
        If state.Exists(key) Then state.Remove key
        state.Add key, entry
    Next change
    WriteStateDictionary state
End Sub

Private Sub WriteStateDictionary(ByVal state As Object)
    Dim stateSheet As Worksheet
    Dim key As Variant
    Dim entry As Object
    Dim rowNumber As Long
    Dim lastRow As Long
    Dim wasProtected As Boolean

    Set stateSheet = ThisWorkbook.Worksheets(STATE_SHEET)
    wasProtected = stateSheet.ProtectContents
    If wasProtected Then stateSheet.Unprotect
    lastRow = stateSheet.Cells(stateSheet.Rows.Count, 1).End(xlUp).Row
    If lastRow >= FIRST_STATE_ROW Then stateSheet.Range("A" & FIRST_STATE_ROW & ":F" & lastRow).ClearContents
    rowNumber = FIRST_STATE_ROW
    For Each key In state.Keys
        Set entry = state(key)
        SafeSetValue stateSheet.Cells(rowNumber, 1), CStr(entry("pointer"))
        SafeSetValue stateSheet.Cells(rowNumber, 2), entry("originalValue")
        SafeSetValue stateSheet.Cells(rowNumber, 3), CStr(entry("originalUnit"))
        SafeSetValue stateSheet.Cells(rowNumber, 4), entry("canonicalValue")
        SafeSetValue stateSheet.Cells(rowNumber, 5), CStr(entry("destination"))
        SafeSetValue stateSheet.Cells(rowNumber, 6), entry("importedAt")
        rowNumber = rowNumber + 1
    Next key
    If wasProtected Then stateSheet.Protect UserInterfaceOnly:=True
End Sub

Private Function StateKey(ByVal pointer As String, ByVal stableId As String) As String
    If Len(stableId) = 0 Then
        StateKey = pointer
    Else
        StateKey = pointer & "#" & stableId
    End If
End Function

Private Sub SafeSetValue(ByVal target As Range, ByVal Value As Variant)
    If CBool(target.HasFormula) Then RaiseExchangeError "Refusing to overwrite formula " & target.Parent.Name & "!" & target.Address(False, False)
    target.Value2 = Value
End Sub

Private Sub ImportPayloadText(ByVal payloadText As String)
    Dim payload As Variant
    Dim root As Object
    Dim mappings As Collection
    Dim changes As Collection
    Dim oldState As Object
    Dim oldBuffer As String
    Dim diagnostics As String
    Dim failureNumber As Long
    Dim failureText As String
    Dim rollbackSucceeded As Boolean

    If Not IsJsonContainerStart(payloadText) Then RaiseExchangeError "Payload root must be a JSON object"
    Set payload = JsonParse(payloadText)
    If TypeName(payload) = "Collection" Then RaiseExchangeError "Payload root must be a JSON object"
    Set root = payload
    ValidatePayload root, diagnostics
    Set mappings = ReadExchangeMap()
    Set oldState = ReadExchangeState()
    oldBuffer = ReadExchangeBuffer()
    WF_EXCHANGE_LAST_ROLLBACK_VERIFIED = False
    On Error GoTo Rollback
    Set changes = BuildChangeSet(root, mappings)
    ApplyChangeSet changes
    WriteExchangeState changes
    SafeSetValue ThisWorkbook.Worksheets(BUFFER_SHEET).Range(BUFFER_PAYLOAD_CELL), payloadText
    Application.CalculateFull
    Exit Sub

Rollback:
    failureNumber = Err.Number
    failureText = Err.Description
    rollbackSucceeded = True
    On Error Resume Next
    If Not changes Is Nothing Then RestoreChangeSet changes
    If Err.Number <> 0 Then rollbackSucceeded = False
    Err.Clear
    WriteStateDictionary oldState
    If Err.Number <> 0 Then rollbackSucceeded = False
    Err.Clear
    SafeSetValue ThisWorkbook.Worksheets(BUFFER_SHEET).Range(BUFFER_PAYLOAD_CELL), oldBuffer
    If Err.Number <> 0 Then rollbackSucceeded = False
    On Error GoTo 0
    If rollbackSucceeded And Not changes Is Nothing Then rollbackSucceeded = WF_ExchangeChangesRestored(changes)
    If rollbackSucceeded Then rollbackSucceeded = WF_ExchangeStatesMatch(ReadExchangeState(), oldState)
    If rollbackSucceeded Then rollbackSucceeded = (ReadExchangeBuffer() = oldBuffer)
    WF_EXCHANGE_LAST_ROLLBACK_VERIFIED = rollbackSucceeded
    If Not rollbackSucceeded Then failureText = failureText & " | ROLLBACK INCOMPLETE"
    Err.Raise failureNumber, MODULE_SOURCE, failureText
End Sub

Private Function WF_ExchangeChangesRestored(ByVal changes As Collection) As Boolean
    Dim change As Variant, target As Range
    WF_ExchangeChangesRestored = False
    On Error GoTo Mismatch
    For Each change In changes
        Set target = change("target")
        If Len(CStr(change("oldFormula"))) > 0 Then
            If CStr(target.Formula) <> CStr(change("oldFormula")) Then Exit Function
        ElseIf Not WF_ExchangeVariantsMatch(target.Value2, change("oldValue")) Then
            Exit Function
        End If
    Next change
    WF_ExchangeChangesRestored = True
Mismatch:
End Function

Private Function WF_ExchangeStatesMatch(ByVal actualState As Object, ByVal expectedState As Object) As Boolean
    Dim key As Variant, actualEntry As Object, expectedEntry As Object, field As Variant
    WF_ExchangeStatesMatch = False
    On Error GoTo Mismatch
    If actualState.Count <> expectedState.Count Then Exit Function
    For Each key In expectedState.Keys
        If Not actualState.Exists(CStr(key)) Then Exit Function
        Set actualEntry = actualState(CStr(key))
        Set expectedEntry = expectedState(CStr(key))
        For Each field In Array("pointer", "originalValue", "originalUnit", "canonicalValue", "destination", "importedAt")
            If Not WF_ExchangeVariantsMatch(actualEntry(CStr(field)), expectedEntry(CStr(field))) Then Exit Function
        Next field
    Next key
    WF_ExchangeStatesMatch = True
Mismatch:
End Function

Private Function WF_ExchangeVariantsMatch(ByVal actualValue As Variant, ByVal expectedValue As Variant) As Boolean
    If IsError(actualValue) Or IsError(expectedValue) Then
        WF_ExchangeVariantsMatch = IsError(actualValue) And IsError(expectedValue) And CStr(actualValue) = CStr(expectedValue)
    ElseIf IsEmpty(actualValue) Or IsEmpty(expectedValue) Then
        WF_ExchangeVariantsMatch = IsEmpty(actualValue) And IsEmpty(expectedValue)
    Else
        WF_ExchangeVariantsMatch = (VarType(actualValue) = VarType(expectedValue) And CStr(actualValue) = CStr(expectedValue))
    End If
End Function

Private Function ReadExchangeBuffer() As String
    ReadExchangeBuffer = CStr(ThisWorkbook.Worksheets(BUFFER_SHEET).Range(BUFFER_PAYLOAD_CELL).Value2)
End Function

Private Sub WriteExchangeBufferStatus(ByVal Status As String, ByVal Diagnostics As String)
    SafeSetValue ThisWorkbook.Worksheets(BUFFER_SHEET).Range(BUFFER_STATUS_CELL), Status
    SafeSetValue ThisWorkbook.Worksheets(BUFFER_SHEET).Range(BUFFER_DIAGNOSTICS_CELL), Diagnostics
End Sub

Private Function ExportPayloadText() As String
    Dim payloadText As String
    Dim payload As Variant
    Dim root As Object
    Dim mappings As Collection
    Dim state As Object
    Dim merged As Object
    Dim diagnostics As String

    payloadText = ReadExchangeBuffer()
    If Not IsJsonContainerStart(payloadText) Then RaiseExchangeError "Exchange Buffer B5 must contain a JSON object before export"
    Set payload = JsonParse(payloadText)
    If TypeName(payload) = "Collection" Then RaiseExchangeError "Exchange Buffer B5 must contain a JSON object"
    Set root = payload
    ValidatePayload root, diagnostics
    Set mappings = ReadExchangeMap()
    Set state = ReadExchangeState()
    Set merged = MergePayload(root, mappings, state)
    ValidatePayload merged, diagnostics
    ExportPayloadText = JsonStringify(merged, 0)
    SafeSetValue ThisWorkbook.Worksheets(BUFFER_SHEET).Range(BUFFER_PAYLOAD_CELL), ExportPayloadText
End Function

Private Function MergePayload(ByVal Payload As Object, ByVal Mappings As Collection, ByVal State As Object) As Object
    Dim mapping As Variant
    Dim targetSheet As Worksheet
    Dim target As Range
    Dim existing As Variant
    Dim encoded As Variant
    Dim found As Boolean
    Dim arrayPointer As String
    Dim fieldName As String
    Dim records As Collection
    Dim recordIndex As Object
    Dim record As Object
    Dim item As Variant
    Dim stableId As String
    Dim rowNumber As Long
    Dim firstRow As Long
    Dim lastRow As Long
    Dim stateEntry As Object

    For Each mapping In Mappings
        Set targetSheet = ThisWorkbook.Worksheets(CStr(mapping("sheet")))
        If StrComp(CStr(mapping("shape")), "Scalar", vbTextCompare) = 0 Then
            Set target = targetSheet.Range(CStr(mapping("address")))
            If ShouldExportCell(target.Value2, mapping) Then
                If PointerIsContainer(Payload, CStr(mapping("pointer"))) Then
                    Set existing = PointerGet(Payload, CStr(mapping("pointer")), found)
                Else
                    existing = PointerGet(Payload, CStr(mapping("pointer")), found)
                End If
                Set stateEntry = StateEntryOrNothing(State, StateKey(CStr(mapping("pointer")), vbNullString))
                If EncodedIsObject(target.Value2, mapping, existing, found) Then
                    Set encoded = EncodeMappedValue(target.Value2, mapping, existing, found, stateEntry)
                    PointerSet Payload, CStr(mapping("pointer")), encoded
                Else
                    encoded = EncodeMappedValue(target.Value2, mapping, existing, found, stateEntry)
                    PointerSet Payload, CStr(mapping("pointer")), encoded
                End If
            End If
        Else
            SplitWildcardPointer CStr(mapping("pointer")), arrayPointer, fieldName
            Set records = EnsurePointerCollection(Payload, arrayPointer)
            Set recordIndex = CreateObject("Scripting.Dictionary")
            recordIndex.CompareMode = vbBinaryCompare
            For Each item In records
                If IsObject(item) And item.Exists("id") Then
                    stableId = CStr(item("id"))
                    If recordIndex.Exists(stableId) Then RaiseExchangeError arrayPointer & " has duplicate stable id " & stableId
                    recordIndex.Add stableId, item
                End If
            Next item

            RangeRowBounds CStr(mapping("address")), firstRow, lastRow
            For rowNumber = firstRow To lastRow
                stableId = CStr(targetSheet.Range(CStr(mapping("idColumn")) & CStr(rowNumber)).Value2)
                If Len(stableId) > 0 Then
                    If recordIndex.Exists(stableId) Then
                        Set record = recordIndex(stableId)
                    Else
                        Set record = CreateObject("Scripting.Dictionary")
                        record.CompareMode = vbBinaryCompare
                        record.Add "id", stableId
                        records.Add record
                        recordIndex.Add stableId, record
                    End If
                    Set target = targetSheet.Range(CStr(mapping("valueColumn")) & CStr(rowNumber))
                    If ShouldExportCell(target.Value2, mapping) Then
                        found = record.Exists(fieldName)
                        If found Then
                            If IsObject(record(fieldName)) Then
                                Set existing = record(fieldName)
                            Else
                                existing = record(fieldName)
                            End If
                        Else
                            existing = Empty
                        End If
                        Set stateEntry = StateEntryOrNothing(State, StateKey(CStr(mapping("pointer")), stableId))
                        If EncodedIsObject(target.Value2, mapping, existing, found) Then
                            Set encoded = EncodeMappedValue(target.Value2, mapping, existing, found, stateEntry)
                        Else
                            encoded = EncodeMappedValue(target.Value2, mapping, existing, found, stateEntry)
                        End If
                        DictionaryPut record, fieldName, encoded
                    End If
                End If
            Next rowNumber
        End If
    Next mapping
    Set MergePayload = Payload
End Function

Private Function ShouldExportCell(ByVal Value As Variant, ByVal mapping As Object) As Boolean
    If IsError(Value) Then RaiseExchangeError "Mapped cell contains an Excel error: " & CStr(mapping("pointer"))
    If IsEmpty(Value) Or IsNull(Value) Then
        ShouldExportCell = False
    ElseIf VarType(Value) = vbString And Len(CStr(Value)) = 0 Then
        ShouldExportCell = (LCase$(CStr(mapping("dataType"))) = "string" And CBool(mapping("required")))
    Else
        ShouldExportCell = True
    End If
End Function

Private Function EncodedIsObject(ByVal cellValue As Variant, ByVal mapping As Object, _
        ByVal existing As Variant, ByVal existingFound As Boolean) As Boolean
    Dim dataType As String
    dataType = LCase$(CStr(mapping("dataType")))
    If existingFound And IsObject(existing) Then
        EncodedIsObject = True
    ElseIf (dataType = "number" Or dataType = "integer") And CStr(mapping("dimension")) <> "unitless" Then
        EncodedIsObject = True
    End If
End Function

Private Function EncodeMappedValue(ByVal cellValue As Variant, ByVal mapping As Object, _
        ByVal existing As Variant, ByVal existingFound As Boolean, ByVal stateEntry As Object) As Variant
    Dim dataType As String
    Dim destinationUnit As String
    Dim wireUnit As String
    Dim canonicalValue As Double
    Dim wireValue As Double
    Dim quantity As Object
    Dim definition As Object

    dataType = LCase$(CStr(mapping("dataType")))
    Select Case dataType
        Case "string", "status"
            EncodeMappedValue = CStr(cellValue)
            Exit Function
        Case "boolean"
            EncodeMappedValue = CBool(cellValue)
            Exit Function
        Case "number", "integer"
            If Not IsNumeric(cellValue) Then RaiseExchangeError "Mapped numeric cell is not numeric: " & CStr(mapping("pointer"))
        Case Else
            RaiseExchangeError "Unsupported mapped data type: " & dataType
    End Select

    If CStr(mapping("dimension")) = "unitless" And Not (existingFound And IsObject(existing)) Then
        If dataType = "integer" Then
            EncodeMappedValue = CLng(cellValue)
        Else
            EncodeMappedValue = CDbl(cellValue)
        End If
        Exit Function
    End If

    destinationUnit = ResolveMappedUnit(mapping)
    canonicalValue = ToSi(CDbl(cellValue), destinationUnit, CStr(mapping("dimension")))
    wireUnit = destinationUnit
    If Not stateEntry Is Nothing Then
        If Len(CStr(stateEntry("originalUnit"))) > 0 And IsNumeric(stateEntry("canonicalValue")) Then
            If NearlyEqual(canonicalValue, CDbl(stateEntry("canonicalValue"))) Then wireUnit = CStr(stateEntry("originalUnit"))
        End If
    End If
    Set definition = LookupUnit(wireUnit, CStr(mapping("dimension")))
    wireValue = FromSi(canonicalValue, wireUnit, CStr(mapping("dimension")))
    Set quantity = CreateObject("Scripting.Dictionary")
    quantity.CompareMode = vbBinaryCompare
    quantity.Add "value", IIf(dataType = "integer", CLng(wireValue), wireValue)
    quantity.Add "unit", wireUnit
    Set EncodeMappedValue = quantity
End Function

Private Function NearlyEqual(ByVal leftValue As Double, ByVal rightValue As Double) As Boolean
    Dim scale As Double
    scale = Abs(leftValue)
    If Abs(rightValue) > scale Then scale = Abs(rightValue)
    If scale < 1# Then scale = 1#
    NearlyEqual = (Abs(leftValue - rightValue) <= scale * 0.000000001)
End Function

Private Function StateEntryOrNothing(ByVal State As Object, ByVal key As String) As Object
    If State.Exists(key) Then Set StateEntryOrNothing = State(key)
End Function

Private Function EnsurePointerCollection(ByVal root As Object, ByVal pointer As String) As Collection
    Dim Value As Variant
    Dim found As Boolean
    Set Value = PointerGet(root, pointer, found)
    If Not found Or TypeName(Value) <> "Collection" Then RaiseExchangeError pointer & " must be an array"
    Set EnsurePointerCollection = Value
End Function

Private Sub PointerSet(ByVal root As Object, ByVal pointer As String, ByVal Value As Variant)
    Dim current As Object
    Dim child As Object
    Dim tokens As Variant
    Dim token As Variant
    Dim decoded As String
    Dim i As Long

    If Left$(pointer, 1) <> "/" Then RaiseExchangeError "Invalid JSON pointer: " & pointer
    Set current = root
    tokens = Split(Mid$(pointer, 2), "/")
    For i = LBound(tokens) To UBound(tokens) - 1
        decoded = DecodePointerToken(CStr(tokens(i)))
        If current.Exists(decoded) Then
            If Not IsObject(current(decoded)) Or TypeName(current(decoded)) = "Collection" Then _
                RaiseExchangeError "Cannot descend into JSON pointer " & pointer
            Set current = current(decoded)
        Else
            Set child = CreateObject("Scripting.Dictionary")
            child.CompareMode = vbBinaryCompare
            current.Add decoded, child
            Set current = child
        End If
    Next i
    decoded = DecodePointerToken(CStr(tokens(UBound(tokens))))
    DictionaryPut current, decoded, Value
End Sub

Private Sub DictionaryPut(ByVal dictionary As Object, ByVal key As String, ByVal Value As Variant)
    If dictionary.Exists(key) Then dictionary.Remove key
    dictionary.Add key, Value
End Sub

Private Sub RaiseExchangeError(ByVal message As String)
    Err.Raise ERR_BASE, MODULE_SOURCE, message
End Sub
