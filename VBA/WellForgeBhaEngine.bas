Attribute VB_Name = "WellForgeBhaEngine"
Option Explicit

Private Const WF_BHA_TIMEOUT_SECONDS As Double = 120#
Private Const WF_BHA_ANALYSIS_UUID As String = "8a547aba-31d2-5aa7-9d52-3284361f0ff8"
Private Const WF_BHA_EXECUTION_MODE As String = "RUST REQUIRED — NO VBA FALLBACK"

Public Sub WF_RunBhaRustEngine()
    Dim executablePath As String, workPath As String, requestPath As String, resultPath As String, bridgePath As String
    Dim request As Object, normalizedHash As String, expectedEngineHash As String
    Dim resultHash As String, rustEngineVersion As String
    Dim exitCode As Long, stdOutText As String, stdErrText As String
    Dim failureNumber As Long, failureDescription As String
    On Error GoTo Failed

    executablePath = ThisWorkbook.Path & Application.PathSeparator & "wellforge-bha.exe"
    If Len(Dir$(executablePath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8700, "WF_RunBhaRustEngine", "ENGINE UNAVAILABLE: " & executablePath
    expectedEngineHash = Trim$(ReadUtf8File(executablePath & ".sha256"))
    If Len(expectedEngineHash) <> 64 Then Err.Raise vbObjectError + 8701, "WF_RunBhaRustEngine", "ENGINE HASH MANIFEST INVALID"
    If StrComp(WF_FileSha256(executablePath), expectedEngineHash, vbTextCompare) <> 0 Then Err.Raise vbObjectError + 8702, "WF_RunBhaRustEngine", "ENGINE HASH MISMATCH"

    workPath = Environ$("TEMP") & Application.PathSeparator & "WellForgeBha" & Application.PathSeparator & Replace$(WF_BHA_ANALYSIS_UUID, "-", "")
    WF_EnsureFolder Environ$("TEMP") & Application.PathSeparator & "WellForgeBha"
    WF_EnsureFolder workPath
    requestPath = workPath & Application.PathSeparator & "request.json"
    resultPath = workPath & Application.PathSeparator & "result.json"
    bridgePath = workPath & Application.PathSeparator & "result.wfbridge"
    Set request = WF_BuildBhaRequest()
    AtomicWriteUtf8 requestPath, JsonStringify(request, 0)

    exitCode = WF_ExecBounded(WF_Quote(executablePath) & " validate --input " & WF_Quote(requestPath), WF_BHA_TIMEOUT_SECONDS, stdOutText, stdErrText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8703, "WF_RunBhaRustEngine", "INVALID REQUEST: " & stdErrText
    normalizedHash = Trim$(stdOutText)
    If Len(normalizedHash) <> 64 Then Err.Raise vbObjectError + 8704, "WF_RunBhaRustEngine", "INVALID REQUEST HASH RETURNED BY ENGINE"

    exitCode = WF_ExecBounded(WF_Quote(executablePath) & " run --input " & WF_Quote(requestPath) & " --output " & WF_Quote(resultPath), WF_BHA_TIMEOUT_SECONDS, stdOutText, stdErrText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8705, "WF_RunBhaRustEngine", "NON-CONVERGED: " & stdErrText
    If Len(Dir$(resultPath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8706, "WF_RunBhaRustEngine", "INVALID RESULT: engine did not create result JSON"
    exitCode = WF_ExecBounded(WF_Quote(executablePath) & " verify-result --input " & WF_Quote(resultPath) & " --request-hash " & WF_Quote(normalizedHash), WF_BHA_TIMEOUT_SECONDS, stdOutText, stdErrText)
    If exitCode <> 0 Or StrComp(Trim$(stdOutText), "valid", vbTextCompare) <> 0 Then Err.Raise vbObjectError + 8707, "WF_RunBhaRustEngine", "INVALID RESULT HASH: " & stdErrText
    exitCode = WF_ExecBounded(WF_Quote(executablePath) & " bridge --input " & WF_Quote(resultPath) & " --output " & WF_Quote(bridgePath) & " --request-hash " & WF_Quote(normalizedHash), WF_BHA_TIMEOUT_SECONDS, stdOutText, stdErrText)
    If exitCode <> 0 Or Len(Dir$(bridgePath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8708, "WF_RunBhaRustEngine", "INVALID RUST BRIDGE: " & stdErrText
    WF_WriteBhaBridge ReadUtf8File(bridgePath), normalizedHash, resultHash, rustEngineVersion

    With ThisWorkbook.Worksheets("Rust Engine")
        .Range("B8").Value2 = requestPath
        .Range("B9").Value2 = resultPath
        .Range("B10").Value2 = normalizedHash
        .Range("B11").Value2 = resultHash
        .Range("B12").Value2 = rustEngineVersion
        .Range("B13").Value2 = "CALCULATED"
    End With
    Exit Sub
Failed:
    failureNumber = Err.Number
    failureDescription = Err.Description
    On Error Resume Next
    ThisWorkbook.Worksheets("Rust Engine").Range("B13").Value2 = "FAILED — LAST ACCEPTED VALUES PRESERVED"
    On Error GoTo 0
    Err.Raise failureNumber, "WF_RunBhaRustEngine", failureDescription
End Sub

Private Function WF_BuildBhaRequest() As Object
    Dim root As Object, sourceList As Collection, trajectoryList As Collection, componentList As Collection, holeList As Collection
    Dim operating As Object, solver As Object, source As Object, station As Object, component As Object, hole As Object
    Dim wsInputs As Worksheet, wsEngine As Worksheet, rowIndex As Long, topMd As Double, bottomMd As Double
    Dim youngPa As Double
    Set wsInputs = ThisWorkbook.Worksheets("Inputs"): Set wsEngine = ThisWorkbook.Worksheets("Rust Engine")
    Set root = CreateObject("Scripting.Dictionary")
    root.Add "contract_version", "1.0.0": root.Add "analysis_id", WF_BHA_ANALYSIS_UUID
    Set sourceList = New Collection
    For rowIndex = 6 To 11
        Set source = CreateObject("Scripting.Dictionary")
        source.Add "uuid", CStr(wsEngine.Cells(rowIndex, 5).Value2)
        source.Add "uri", CStr(wsEngine.Cells(rowIndex, 6).Value2)
        source.Add "object_type", CStr(wsEngine.Cells(rowIndex, 4).Value2)
        source.Add "content_hash", CStr(wsEngine.Cells(rowIndex, 7).Value2)
        source.Add "citation_name", CStr(wsEngine.Cells(rowIndex, 8).Value2)
        source.Add "source_system", CStr(wsEngine.Cells(rowIndex, 9).Value2)
        sourceList.Add source
    Next rowIndex
    root.Add "sources", sourceList
    Set trajectoryList = New Collection
    Set station = CreateObject("Scripting.Dictionary"): station.Add "md_m", 0#: station.Add "inclination_rad", CDbl(wsInputs.Range("B15").Value2) * 3.14159265358979 / 180#: station.Add "azimuth_rad", 0#: trajectoryList.Add station

    youngPa = WF_BhaYoungModulusPa(CDbl(wsInputs.Range("B7").Value2), CStr(wsInputs.Range("C7").Value2))
    Set componentList = New Collection: topMd = 0#
    For rowIndex = 6 To 11
        bottomMd = topMd + CDbl(wsInputs.Cells(rowIndex, 5).Value2)
        Set component = CreateObject("Scripting.Dictionary")
        component.Add "id", CStr(wsEngine.Cells(rowIndex, 11).Value2)
        component.Add "name", CStr(wsInputs.Cells(rowIndex, 4).Value2)
        component.Add "representation", "beam"
        component.Add "top_md_m", topMd: component.Add "bottom_md_m", bottomMd
        component.Add "od_m", CDbl(wsInputs.Cells(rowIndex, 6).Value2)
        component.Add "id_m", CDbl(wsInputs.Cells(rowIndex, 7).Value2)
        component.Add "youngs_modulus_pa", youngPa
        component.Add "density_kg_m3", CDbl(wsInputs.Range("B8").Value2)
        componentList.Add component: topMd = bottomMd
    Next rowIndex
    Set station = CreateObject("Scripting.Dictionary"): station.Add "md_m", topMd: station.Add "inclination_rad", CDbl(wsInputs.Range("B15").Value2) * 3.14159265358979 / 180#: station.Add "azimuth_rad", 0#: trajectoryList.Add station
    root.Add "trajectory", trajectoryList
    root.Add "components", componentList
    Set holeList = New Collection: Set hole = CreateObject("Scripting.Dictionary")
    hole.Add "top_md_m", 0#: hole.Add "bottom_md_m", topMd: hole.Add "diameter_m", CDbl(wsInputs.Range("B11").Value2)
    holeList.Add hole: root.Add "hole", holeList
    Set operating = CreateObject("Scripting.Dictionary")
    operating.Add "wob_n", CDbl(wsInputs.Range("B10").Value2): operating.Add "rpm", CDbl(wsInputs.Range("B5").Value2)
    operating.Add "fluid_density_kg_m3", CDbl(wsInputs.Range("B14").Value2): root.Add "operating", operating
    Set solver = CreateObject("Scripting.Dictionary")
    solver.Add "max_element_length_m", 0.5: solver.Add "max_iterations", 80
    solver.Add "residual_tolerance", 0.00000001: solver.Add "contact_penalty_n_m", 1000000000#
    solver.Add "requested_modes", 8: root.Add "solver", solver
    Set WF_BuildBhaRequest = root
End Function

Private Sub WF_WriteBhaBridge(ByVal bridgeText As String, ByVal normalizedRequestHash As String, ByRef resultHash As String, ByRef rustEngineVersion As String)
    Dim ws As Worksheet, wsResults As Worksheet, lines As Variant, fields As Variant, line As Variant
    Dim staticCount As Long, modeCount As Long, frfCount As Long, campbellCount As Long
    Dim shapeCount(1 To 3) As Long, modeNumber As Long, rowIndex As Long
    Dim minClearance As Double, peakStress As Double, minMargin As Double, value As Double
    Dim firstFrequency As Double, headerSeen As Boolean
    Set ws = ThisWorkbook.Worksheets("Rust Calc")
    Set wsResults = ThisWorkbook.Worksheets("Rust Engine Results")
    lines = Split(Replace$(bridgeText, vbCr, vbNullString), vbLf)
    WF_ValidateBhaBridge lines, normalizedRequestHash, resultHash, rustEngineVersion
    ws.Range("A6:K505,L6:O25,L251:O750,A41:C440,E41:H640").ClearContents
    minClearance = 1E+99: minMargin = 1E+99
    wsResults.Range("A13:D32").ClearContents
    For Each line In lines
        If Len(CStr(line)) > 0 Then
            fields = Split(CStr(line), vbTab)
            Select Case CStr(fields(0))
                Case "H"
                    If UBound(fields) <> 7 Or CStr(fields(1)) <> "1.0.0" Then Err.Raise vbObjectError + 8710, "WF_WriteBhaBridge", "INVALID BRIDGE CONTRACT"
                    If CStr(fields(2)) <> WF_BHA_ANALYSIS_UUID Then Err.Raise vbObjectError + 8711, "WF_WriteBhaBridge", "INVALID BRIDGE ANALYSIS ID"
                    If StrComp(CStr(fields(3)), normalizedRequestHash, vbTextCompare) <> 0 Then Err.Raise vbObjectError + 8713, "WF_WriteBhaBridge", "INVALID BRIDGE REQUEST HASH"
                    If LCase$(CStr(fields(7))) <> "true" Then Err.Raise vbObjectError + 8714, "WF_WriteBhaBridge", "NON-CONVERGED BRIDGE"
                    resultHash = CStr(fields(4)): rustEngineVersion = CStr(fields(5)): headerSeen = True
                Case "S"
                    staticCount = staticCount + 1: If staticCount > 500 Then Err.Raise vbObjectError + 8716, "WF_WriteBhaBridge", "STATIC RESULT CAPACITY EXCEEDED"
                    rowIndex = 5 + staticCount
                    ws.Cells(rowIndex, 1).Resize(1, 9).Value2 = WF_BridgeRow(fields, 1, 9)
                    value = WF_BridgeNumber(fields(7)): ws.Cells(rowIndex, 10).Value2 = IIf(value < 0#, "OVERLAP INDICATION", "CLEARANCE")
                    ws.Cells(rowIndex, 11).Value2 = -WF_BridgeNumber(fields(6))
                    If value < minClearance Then minClearance = value
                    value = WF_BridgeNumber(fields(9)): If value > peakStress Then peakStress = value
                Case "M"
                    modeCount = modeCount + 1: If modeCount > 20 Then Err.Raise vbObjectError + 8717, "WF_WriteBhaBridge", "MODE RESULT CAPACITY EXCEEDED"
                    rowIndex = 5 + modeCount: modeNumber = CLng(fields(1))
                    ws.Cells(rowIndex, 12).Value2 = modeNumber: ws.Cells(rowIndex, 13).Value2 = WF_BridgeNumber(fields(2)): ws.Cells(rowIndex, 14).Value2 = WF_BridgeNumber(fields(3)): ws.Cells(rowIndex, 15).Value2 = "CALCULATED"
                    wsResults.Cells(12 + modeCount, 1).Value2 = modeNumber: wsResults.Cells(12 + modeCount, 2).Value2 = WF_BridgeNumber(fields(2)): wsResults.Cells(12 + modeCount, 3).Value2 = WF_BridgeNumber(fields(3)): wsResults.Cells(12 + modeCount, 4).Value2 = "CALCULATED"
                    If modeCount = 1 Then firstFrequency = WF_BridgeNumber(fields(2))
                Case "P"
                    modeNumber = CLng(fields(1))
                    If modeNumber >= 1 And modeNumber <= 3 Then
                        shapeCount(modeNumber) = shapeCount(modeNumber) + 1: If shapeCount(modeNumber) > 500 Then Err.Raise vbObjectError + 8718, "WF_WriteBhaBridge", "MODE SHAPE CAPACITY EXCEEDED"
                        rowIndex = 250 + shapeCount(modeNumber): ws.Cells(rowIndex, 12).Value2 = WF_BridgeNumber(fields(2)): ws.Cells(rowIndex, 12 + modeNumber).Value2 = WF_BridgeNumber(fields(3))
                    End If
                Case "F"
                    frfCount = frfCount + 1: If frfCount > 400 Then Err.Raise vbObjectError + 8719, "WF_WriteBhaBridge", "FRF CAPACITY EXCEEDED"
                    ws.Cells(40 + frfCount, 1).Resize(1, 3).Value2 = WF_BridgeRow(fields, 1, 3)
                Case "C"
                    campbellCount = campbellCount + 1: If campbellCount > 600 Then Err.Raise vbObjectError + 8723, "WF_WriteBhaBridge", "CAMPBELL CAPACITY EXCEEDED"
                    ws.Cells(40 + campbellCount, 5).Resize(1, 4).Value2 = WF_BridgeRow(fields, 1, 4)
                    value = WF_BridgeNumber(fields(4)): If value < minMargin Then minMargin = value
                Case Else
                    Err.Raise vbObjectError + 8724, "WF_WriteBhaBridge", "UNKNOWN BRIDGE RECORD: " & CStr(fields(0))
            End Select
        End If
    Next line
    If Not headerSeen Or Len(resultHash) <> 64 Or staticCount < 2 Or modeCount < 1 Then Err.Raise vbObjectError + 8715, "WF_WriteBhaBridge", "INCOMPLETE BRIDGE RESULTS"
    wsResults.Range("B6").Value2 = minClearance: wsResults.Range("D6").Value2 = IIf(minClearance < 0#, "REVIEW", "CLEAR")
    wsResults.Range("B7").Value2 = peakStress: wsResults.Range("B8").Value2 = firstFrequency: wsResults.Range("B9").Value2 = minMargin
End Sub

Private Sub WF_ValidateBhaBridge(ByVal lines As Variant, ByVal normalizedRequestHash As String, _
                                 ByRef resultHash As String, ByRef rustEngineVersion As String)
    Dim line As Variant, fields As Variant, recordType As String
    Dim staticCount As Long, modeCount As Long, frfCount As Long, campbellCount As Long
    Dim shapeCount(1 To 3) As Long, modeNumber As Long, headerSeen As Boolean
    For Each line In lines
        If Len(CStr(line)) > 0 Then
            fields = Split(CStr(line), vbTab)
            recordType = CStr(fields(0))
            Select Case recordType
                Case "H"
                    If UBound(fields) <> 7 Or CStr(fields(1)) <> "1.0.0" Then _
                        Err.Raise vbObjectError + 8710, "WF_ValidateBhaBridge", "INVALID BRIDGE CONTRACT"
                    If headerSeen Then Err.Raise vbObjectError + 8725, "WF_ValidateBhaBridge", "DUPLICATE BRIDGE HEADER"
                    If CStr(fields(2)) <> WF_BHA_ANALYSIS_UUID Then _
                        Err.Raise vbObjectError + 8711, "WF_ValidateBhaBridge", "INVALID BRIDGE ANALYSIS ID"
                    If StrComp(CStr(fields(3)), normalizedRequestHash, vbTextCompare) <> 0 Then _
                        Err.Raise vbObjectError + 8713, "WF_ValidateBhaBridge", "INVALID BRIDGE REQUEST HASH"
                    If LCase$(CStr(fields(7))) <> "true" Then _
                        Err.Raise vbObjectError + 8714, "WF_ValidateBhaBridge", "NON-CONVERGED BRIDGE"
                    resultHash = CStr(fields(4))
                    rustEngineVersion = CStr(fields(5))
                    headerSeen = True
                Case "S"
                    If UBound(fields) <> 9 Then Err.Raise vbObjectError + 8726, "WF_ValidateBhaBridge", "INVALID STATIC RECORD"
                    staticCount = staticCount + 1
                    If staticCount > 500 Then Err.Raise vbObjectError + 8716, "WF_ValidateBhaBridge", "STATIC RESULT CAPACITY EXCEEDED"
                Case "M"
                    If UBound(fields) <> 3 Then Err.Raise vbObjectError + 8727, "WF_ValidateBhaBridge", "INVALID MODE RECORD"
                    If Not IsNumeric(fields(1)) Then Err.Raise vbObjectError + 8731, "WF_ValidateBhaBridge", "INVALID MODE RECORD NUMBER"
                    modeCount = modeCount + 1
                    If modeCount > 20 Then Err.Raise vbObjectError + 8717, "WF_ValidateBhaBridge", "MODE RESULT CAPACITY EXCEEDED"
                Case "P"
                    If UBound(fields) <> 3 Then Err.Raise vbObjectError + 8728, "WF_ValidateBhaBridge", "INVALID MODE SHAPE RECORD"
                    If Not IsNumeric(fields(1)) Then Err.Raise vbObjectError + 8732, "WF_ValidateBhaBridge", "INVALID MODE SHAPE RECORD NUMBER"
                    modeNumber = CLng(fields(1))
                    If modeNumber >= 1 And modeNumber <= 3 Then
                        shapeCount(modeNumber) = shapeCount(modeNumber) + 1
                        If shapeCount(modeNumber) > 500 Then _
                            Err.Raise vbObjectError + 8718, "WF_ValidateBhaBridge", "MODE SHAPE CAPACITY EXCEEDED"
                    End If
                Case "F"
                    If UBound(fields) <> 3 Then Err.Raise vbObjectError + 8729, "WF_ValidateBhaBridge", "INVALID FRF RECORD"
                    frfCount = frfCount + 1
                    If frfCount > 400 Then Err.Raise vbObjectError + 8719, "WF_ValidateBhaBridge", "FRF CAPACITY EXCEEDED"
                Case "C"
                    If UBound(fields) <> 4 Then Err.Raise vbObjectError + 8730, "WF_ValidateBhaBridge", "INVALID CAMPBELL RECORD"
                    campbellCount = campbellCount + 1
                    If campbellCount > 600 Then Err.Raise vbObjectError + 8723, "WF_ValidateBhaBridge", "CAMPBELL CAPACITY EXCEEDED"
                Case Else
                    Err.Raise vbObjectError + 8724, "WF_ValidateBhaBridge", "UNKNOWN BRIDGE RECORD: " & recordType
            End Select
        End If
    Next line
    If Not headerSeen Or Len(resultHash) <> 64 Or staticCount < 2 Or modeCount < 1 Or _
       frfCount < 1 Or campbellCount < 1 Then _
        Err.Raise vbObjectError + 8715, "WF_ValidateBhaBridge", "INCOMPLETE BRIDGE RESULTS"
End Sub

Private Function WF_BridgeRow(ByVal fields As Variant, ByVal firstIndex As Long, ByVal fieldCount As Long) As Variant
    Dim values() As Variant, index As Long
    ReDim values(1 To 1, 1 To fieldCount)
    For index = 1 To fieldCount
        values(1, index) = WF_BridgeNumber(fields(firstIndex + index - 1))
    Next index
    WF_BridgeRow = values
End Function

Private Function WF_BridgeNumber(ByVal textValue As Variant) As Double
    WF_BridgeNumber = Val(CStr(textValue))
End Function

Private Function WF_ExecBounded(ByVal commandLine As String, ByVal timeoutSeconds As Double, ByRef stdOutText As String, ByRef stdErrText As String) As Long
    Dim shell As Object, process As Object, started As Double
    Set shell = CreateObject("WScript.Shell"): Set process = shell.Exec(commandLine): started = Timer
    Do While process.Status = 0
        DoEvents
        If WF_ElapsedSeconds(started) > timeoutSeconds Then process.Terminate: Err.Raise vbObjectError + 8720, "WF_ExecBounded", "ENGINE TIMEOUT"
    Loop
    stdOutText = process.StdOut.ReadAll: stdErrText = process.StdErr.ReadAll: WF_ExecBounded = process.ExitCode
End Function

Private Function WF_FileSha256(ByVal filePath As String) As String
    Dim commandLine As String, outputText As String, errorText As String, exitCode As Long
    commandLine = "powershell.exe -NoProfile -NonInteractive -Command " & WF_Quote("(Get-FileHash -Algorithm SHA256 -LiteralPath '" & Replace$(filePath, "'", "''") & "').Hash")
    exitCode = WF_ExecBounded(commandLine, 30#, outputText, errorText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8721, "WF_FileSha256", "Unable to hash engine: " & errorText
    WF_FileSha256 = LCase$(Trim$(outputText))
End Function

Private Function WF_Quote(ByVal value As String) As String
    WF_Quote = Chr$(34) & Replace$(value, Chr$(34), Chr$(34) & Chr$(34)) & Chr$(34)
End Function

Private Function WF_ElapsedSeconds(ByVal started As Double) As Double
    WF_ElapsedSeconds = Timer - started
    If WF_ElapsedSeconds < 0# Then WF_ElapsedSeconds = WF_ElapsedSeconds + 86400#
End Function

Private Sub WF_EnsureFolder(ByVal folderPath As String)
    If Len(Dir$(folderPath, vbDirectory)) = 0 Then MkDir folderPath
End Sub

Private Function WF_BhaYoungModulusPa(ByVal value As Double, ByVal unitSymbol As String) As Double
    Select Case unitSymbol
        Case "GPa": WF_BhaYoungModulusPa = value * 1000000000#
        Case "MPa": WF_BhaYoungModulusPa = value * 1000000#
        Case "Pa": WF_BhaYoungModulusPa = value
        Case "Mpsi": WF_BhaYoungModulusPa = value * 6894757293.168
        Case "psi": WF_BhaYoungModulusPa = value * 6894.757293168
        Case Else: Err.Raise vbObjectError + 8722, "WF_BhaYoungModulusPa", "Unsupported modulus unit: " & unitSymbol
    End Select
End Function
