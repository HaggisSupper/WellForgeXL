Attribute VB_Name = "WellForgeCore"
Option Explicit

Public Const WF_ENGINE_VERSION As String = "2.0.0-vba"
Public WF_Busy As Boolean

Private Const WF_PI As Double = 3.14159265358979
Private Const WF_TWO_PI As Double = 6.28318530717959

Public Sub WellForge_InitializeWorkbook()
    Dim oldCalc As XlCalculation
    Dim oldEvents As Boolean
    Dim oldScreen As Boolean

    If WF_Busy Then Exit Sub
    WF_Busy = True
    oldCalc = Application.Calculation
    oldEvents = Application.EnableEvents
    oldScreen = Application.ScreenUpdating
    On Error GoTo Failed

    Application.Calculation = xlCalculationManual
    Application.EnableEvents = False
    Application.ScreenUpdating = False
    Application.CalculateFullRebuild
    WF_FreezeAllFormulas
    WF_ReplacePocLanguage
    WF_InstallControls
    WF_UpdateUnitMap
    WF_DispatchModel WF_ModelKind()
    WF_RefreshCharts
    WF_WriteEngineStatus "READY", WF_ModelKind() & " initialized"

Cleanup:
    Application.Calculation = oldCalc
    Application.EnableEvents = oldEvents
    Application.ScreenUpdating = oldScreen
    WF_Busy = False
    Exit Sub
Failed:
    WF_WriteEngineStatus "ERROR", Err.Description
    MsgBox "WellForge VBA initialization failed:" & vbCrLf & Err.Description, vbCritical, "WellForge"
    Resume Cleanup
End Sub

Public Sub WellForge_CalculateAll(Optional ByVal ShowCompletion As Boolean = True)
    Dim oldCalc As XlCalculation
    Dim oldEvents As Boolean
    Dim oldScreen As Boolean
    Dim started As Double
    Dim model As String

    If WF_Busy Then Exit Sub
    WF_Busy = True
    oldCalc = Application.Calculation
    oldEvents = Application.EnableEvents
    oldScreen = Application.ScreenUpdating
    started = Timer
    On Error GoTo Failed

    Application.Calculation = xlCalculationManual
    Application.EnableEvents = False
    Application.ScreenUpdating = False
    WF_UpdateUnitMap
    model = WF_ModelKind()

    WF_DispatchModel model

    WF_RefreshCharts
    WF_WriteEngineStatus "READY", model & " calculated in " & Format$(Timer - started, "0.00") & " s"
    If ShowCompletion Then MsgBox "Calculation complete." & vbCrLf & model & " | Engine " & WF_ENGINE_VERSION, vbInformation, "WellForge"

Cleanup:
    Application.Calculation = oldCalc
    Application.EnableEvents = oldEvents
    Application.ScreenUpdating = oldScreen
    WF_Busy = False
    Exit Sub
Failed:
    WF_WriteEngineStatus "ERROR", model & ": " & Err.Description
    MsgBox "WellForge calculation failed:" & vbCrLf & Err.Description, vbCritical, "WellForge"
    Resume Cleanup
End Sub

Public Sub WellForge_BuildInitialize()
    Dim oldCalc As XlCalculation
    Dim oldEvents As Boolean
    Dim oldScreen As Boolean
    Dim failureNumber As Long
    Dim failureDescription As String
    Dim model As String

    If WF_Busy Then Exit Sub
    WF_Busy = True
    oldCalc = Application.Calculation: oldEvents = Application.EnableEvents: oldScreen = Application.ScreenUpdating
    On Error GoTo Failed
    Application.Calculation = xlCalculationManual: Application.EnableEvents = False: Application.ScreenUpdating = False
    Application.CalculateFullRebuild
    WF_FreezeAllFormulas
    WF_ReplacePocLanguage
    WF_InstallControls
    WF_UpdateUnitMap
    model = WF_ModelKind()
    WF_DispatchModel model
    WF_RefreshCharts
    WF_WriteEngineStatus "READY", model & " compiled and initialized"
Cleanup:
    Application.Calculation = oldCalc: Application.EnableEvents = oldEvents: Application.ScreenUpdating = oldScreen: WF_Busy = False
    If failureNumber <> 0 Then Err.Raise failureNumber, "WellForge_BuildInitialize", failureDescription
    Exit Sub
Failed:
    failureNumber = Err.Number: failureDescription = Err.Description
    Resume Cleanup
End Sub

Private Sub WF_DispatchModel(ByVal model As String)
    Select Case model
        Case "API7G": WF_CalcAPI7G
        Case "HYDRAULICS": WF_CalcHydraulics
        Case "TORQUE_DRAG": WF_CalcTorqueDrag
        Case "BHA": WF_RunBhaRustEngine
        Case "DIRECTIONAL": WF_RunTrajectoryRustEngine
        Case Else: Err.Raise vbObjectError + 8200, "WF_DispatchModel", "Unable to identify workbook analysis type"
    End Select
End Sub

Public Sub WellForge_ValidateModel()
    On Error GoTo Failed
    WellForge_CalculateAll False
    MsgBox "Calculation completed without a VBA runtime error." & vbCrLf & _
           "Review the Summary and Checks sheets for engineering-screening exceptions.", _
           vbInformation, "WellForge validation"
    Exit Sub
Failed:
    MsgBox Err.Description, vbCritical, "WellForge validation"
End Sub

Public Sub WellForge_ResetOutputs()
    Dim answer As VbMsgBoxResult
    answer = MsgBox("Recalculate all outputs from the current input cells?", vbQuestion + vbYesNo, "WellForge")
    If answer = vbYes Then WellForge_CalculateAll
End Sub

Public Sub WF_HandleSheetChange(ByVal SheetName As String, ByVal ChangedCells As Long)
    If WF_Busy Then Exit Sub
    If ChangedCells > 1000 Then Exit Sub
    Select Case SheetName
        Case "Chart Settings", "Observed Data", "Flow Cases"
            WellForge_CalculateAll False
        Case "Inputs", "Unit Map", "Survey", "Plan", "Targets", "Slide Performance", "Formation Tops", _
             "Tubular Catalog", "Load Cases", "Wellbore", "Drillstring", "Operation Cases", "BHA Assembly", "Fluid Model"
            WellForge_CalculateAll False
    End Select
End Sub

Public Function WF_NearestDepthRow(ByVal SheetName As String, ByVal FirstRow As Long, ByVal RowCount As Long, ByVal DepthColumn As Long, ByVal TargetDepthSI As Double) As Long
    Dim ws As Worksheet, rowOffset As Long
    Dim candidate As Double, bestDifference As Double, difference As Double
    If RowCount <= 0 Then Err.Raise vbObjectError + 8220, "WF_NearestDepthRow", "Depth lookup contains no rows"
    Set ws = ThisWorkbook.Worksheets(SheetName)
    bestDifference = 1E+99
    For rowOffset = 0 To RowCount - 1
        If IsNumeric(ws.Cells(FirstRow + rowOffset, DepthColumn).Value2) Then
            candidate = CDbl(ws.Cells(FirstRow + rowOffset, DepthColumn).Value2)
            difference = Abs(candidate - TargetDepthSI)
            If difference < bestDifference Then
                bestDifference = difference
                WF_NearestDepthRow = FirstRow + rowOffset
            End If
        End If
    Next rowOffset
    If WF_NearestDepthRow = 0 Then Err.Raise vbObjectError + 8221, "WF_NearestDepthRow", "No numeric depth was found"
End Function

Public Sub WellForge_VisualizationSelfTest()
    Dim model As String
    If Not WF_SheetExists("Chart Settings") Then Err.Raise vbObjectError + 8222, "WellForge_VisualizationSelfTest", "Chart Settings is missing"
    If StrComp(CStr(ThisWorkbook.Worksheets("Chart Settings").Range("A6").Value2), "Selected MD", vbTextCompare) <> 0 Then _
        Err.Raise vbObjectError + 8223, "WellForge_VisualizationSelfTest", "Selected MD chart setting is missing"
    model = WF_ModelKind()
    Select Case model
        Case "TORQUE_DRAG"
            WF_AssertDashboard "Engineering Dashboard", 4
            If Not WF_SheetExists("Observed Data") Then Err.Raise vbObjectError + 8224, "WellForge_VisualizationSelfTest", "Observed Data is missing"
            WF_AssertSeriesCount "Engineering Dashboard", 1, 10
            WF_AssertSeriesCount "Engineering Dashboard", 2, 5
            WF_AssertSeriesCount "Engineering Dashboard", 4, 3
        Case "HYDRAULICS"
            WF_AssertDashboard "Hydraulics Dashboard", 4
            If Not WF_SheetExists("Flow Cases") Then Err.Raise vbObjectError + 8225, "WellForge_VisualizationSelfTest", "Flow Cases is missing"
            WF_AssertSeriesCount "Hydraulics Dashboard", 2, 5
            WF_AssertSeriesCount "Hydraulics Dashboard", 3, 4
        Case "BHA"
            WF_AssertDashboard "BHA Geometry View", 4
            WF_AssertDashboard "Vibration Modes", 2
            WF_AssertSeriesCount "BHA Geometry View", 1, 7
            WF_AssertSeriesCount "BHA Geometry View", 2, 2
            WF_AssertSeriesCount "BHA Geometry View", 4, 2
            If InStr(1, CStr(ThisWorkbook.Worksheets("BHA Geometry View").Range("A5").Value2), "not solved contact or reaction force", vbTextCompare) = 0 Then _
                Err.Raise vbObjectError + 8230, "WellForge_VisualizationSelfTest", "BHA geometry limitation statement is missing"
        Case "API7G", "DIRECTIONAL"
            ' Discipline-specific charts retain their established topology; Chart Settings is the persisted contract.
        Case Else
            Err.Raise vbObjectError + 8226, "WellForge_VisualizationSelfTest", "Unknown workbook model"
    End Select
End Sub

Private Sub WF_AssertDashboard(ByVal SheetName As String, ByVal MinimumCharts As Long)
    If Not WF_SheetExists(SheetName) Then Err.Raise vbObjectError + 8227, "WF_AssertDashboard", SheetName & " is missing"
    If ThisWorkbook.Worksheets(SheetName).ChartObjects.Count < MinimumCharts Then _
        Err.Raise vbObjectError + 8228, "WF_AssertDashboard", SheetName & " has fewer than " & CStr(MinimumCharts) & " charts"
End Sub

Private Sub WF_AssertSeriesCount(ByVal SheetName As String, ByVal ChartIndex As Long, ByVal MinimumSeries As Long)
    Dim countSeries As Long
    countSeries = ThisWorkbook.Worksheets(SheetName).ChartObjects(ChartIndex).Chart.SeriesCollection.Count
    If countSeries < MinimumSeries Then Err.Raise vbObjectError + 8229, "WF_AssertSeriesCount", SheetName & " chart " & CStr(ChartIndex) & " is missing required series"
End Sub

Public Function WF_ModelKind() As String
    If WF_SheetExists("Tubular Catalog") Then WF_ModelKind = "API7G": Exit Function
    If WF_SheetExists("Fluid Model") Then WF_ModelKind = "HYDRAULICS": Exit Function
    If WF_SheetExists("Wellbore") Then WF_ModelKind = "TORQUE_DRAG": Exit Function
    If WF_SheetExists("BHA Assembly") Then WF_ModelKind = "BHA": Exit Function
    If WF_SheetExists("Plan") And WF_SheetExists("Targets") Then WF_ModelKind = "DIRECTIONAL": Exit Function
End Function

Public Function WF_SheetExists(ByVal SheetName As String) As Boolean
    Dim ws As Worksheet
    On Error Resume Next
    Set ws = ThisWorkbook.Worksheets(SheetName)
    WF_SheetExists = Not ws Is Nothing
    On Error GoTo 0
End Function

Public Function WF_Num(ByVal SheetName As String, ByVal Address As String, Optional ByVal DefaultValue As Double = 0#) As Double
    Dim value As Variant
    value = ThisWorkbook.Worksheets(SheetName).Range(Address).Value2
    If IsError(value) Or IsEmpty(value) Or Len(Trim$(CStr(value))) = 0 Or Not IsNumeric(value) Then
        WF_Num = DefaultValue
    Else
        WF_Num = CDbl(value)
    End If
End Function

Public Function WF_Str(ByVal SheetName As String, ByVal Address As String, Optional ByVal DefaultValue As String = "") As String
    Dim value As Variant
    value = ThisWorkbook.Worksheets(SheetName).Range(Address).Value2
    If IsError(value) Or IsEmpty(value) Then WF_Str = DefaultValue Else WF_Str = CStr(value)
End Function

Public Function WF_Clamp(ByVal Value As Double, ByVal Minimum As Double, ByVal Maximum As Double) As Double
    If Value < Minimum Then Value = Minimum
    If Value > Maximum Then Value = Maximum
    WF_Clamp = Value
End Function

Public Function WF_Mod2Pi(ByVal Value As Double) As Double
    WF_Mod2Pi = Value - WF_TWO_PI * Int(Value / WF_TWO_PI)
    If WF_Mod2Pi < 0# Then WF_Mod2Pi = WF_Mod2Pi + WF_TWO_PI
End Function

Public Function WF_DegToRad(ByVal Value As Double) As Double
    WF_DegToRad = Value * WF_PI / 180#
End Function

Public Function WF_RadToDeg(ByVal Value As Double) As Double
    WF_RadToDeg = Value * 180# / WF_PI
End Function

Public Function WF_ToSI(ByVal Value As Double, ByVal UnitName As String) As Double
    Select Case LCase$(Trim$(UnitName))
        Case "m", "rad", "rad/m", "pa", "n", "n-m", "n*m", "kg/m3", "m3/s", "m/s", "hz", "1": WF_ToSI = Value
        Case "ft": WF_ToSI = Value * 0.3048
        Case "in": WF_ToSI = Value * 0.0254
        Case "mm": WF_ToSI = Value * 0.001
        Case "deg": WF_ToSI = WF_DegToRad(Value)
        Case "deg/100ft": WF_ToSI = Value * 0.000572957795130823
        Case "deg/30m": WF_ToSI = Value * 0.000581776417331443
        Case "psi": WF_ToSI = Value * 6894.757293168
        Case "kpa": WF_ToSI = Value * 1000#
        Case "mpa": WF_ToSI = Value * 1000000#
        Case "gpa": WF_ToSI = Value * 1000000000#
        Case "mpsi": WF_ToSI = Value * 6894757293.168
        Case "lbf": WF_ToSI = Value * 4.4482301
        Case "klbf": WF_ToSI = Value * 4448.2301
        Case "kn": WF_ToSI = Value * 1000#
        Case "ft-lbf": WF_ToSI = Value * 1.3558179483314
        Case "kn-m", "kn*m": WF_ToSI = Value * 1000#
        Case "gpm": WF_ToSI = Value * 0.0000630901964
        Case "l/min": WF_ToSI = Value / 60000#
        Case "ppg": WF_ToSI = Value * 119.826427316
        Case "cp": WF_ToSI = Value * 0.001
        Case Else: Err.Raise vbObjectError + 8201, "WF_ToSI", "Unsupported unit: " & UnitName
    End Select
End Function

Public Function WF_UnitFactor(ByVal DomainName As String) As Double
    Dim rowIndex As Long
    rowIndex = WF_UnitRow(DomainName)
    If rowIndex = 0 Then Err.Raise vbObjectError + 8202, "WF_UnitFactor", "Unknown unit domain: " & DomainName
    WF_UnitFactor = CDbl(ThisWorkbook.Worksheets("Unit Map").Cells(rowIndex, 9).Value2)
End Function

Public Function WF_UnitLabel(ByVal DomainName As String) As String
    Dim rowIndex As Long
    rowIndex = WF_UnitRow(DomainName)
    If rowIndex = 0 Then Err.Raise vbObjectError + 8203, "WF_UnitLabel", "Unknown unit domain: " & DomainName
    WF_UnitLabel = CStr(ThisWorkbook.Worksheets("Unit Map").Cells(rowIndex, 8).Value2)
End Function

Private Function WF_UnitRow(ByVal DomainName As String) As Long
    Dim rowIndex As Long
    For rowIndex = 8 To 40
        If StrComp(CStr(ThisWorkbook.Worksheets("Unit Map").Cells(rowIndex, 1).Value2), DomainName, vbTextCompare) = 0 Then
            WF_UnitRow = rowIndex
            Exit Function
        End If
    Next rowIndex
End Function

Public Sub WF_UpdateUnitMap()
    Dim ws As Worksheet
    Dim systemName As String
    Dim customName As String
    Dim rowIndex As Long
    Dim choiceColumn As Long

    Set ws = ThisWorkbook.Worksheets("Unit Map")
    systemName = Trim$(CStr(ws.Range("B5").Value2))
    If systemName <> "SI" And systemName <> "Imperial" And systemName <> "Mixed" And systemName <> "Custom" Then
        Err.Raise vbObjectError + 8204, "WF_UpdateUnitMap", "Invalid display system: " & systemName
    End If

    For rowIndex = 8 To 40
        If Len(CStr(ws.Cells(rowIndex, 1).Value2)) = 0 Then Exit For
        If systemName = "Custom" Then customName = Trim$(CStr(ws.Cells(rowIndex, 10).Value2)) Else customName = systemName
        Select Case customName
            Case "SI": choiceColumn = 2: ws.Cells(rowIndex, 9).Value2 = 1#
            Case "Imperial": choiceColumn = 3: ws.Cells(rowIndex, 9).Value2 = ws.Cells(rowIndex, 5).Value2
            Case "Mixed": choiceColumn = 4: ws.Cells(rowIndex, 9).Value2 = ws.Cells(rowIndex, 6).Value2
            Case Else: Err.Raise vbObjectError + 8205, "WF_UpdateUnitMap", "Invalid custom unit choice at row " & CStr(rowIndex)
        End Select
        ws.Cells(rowIndex, 8).Value2 = ws.Cells(rowIndex, choiceColumn).Value2
    Next rowIndex
End Sub

Public Sub WF_FreezeAllFormulas()
    Dim ws As Worksheet
    Dim formulaCells As Range
    For Each ws In ThisWorkbook.Worksheets
        Set formulaCells = Nothing
        On Error Resume Next
        Set formulaCells = ws.UsedRange.SpecialCells(xlCellTypeFormulas)
        On Error GoTo 0
        If Not formulaCells Is Nothing Then formulaCells.Value2 = formulaCells.Value2
    Next ws
End Sub

Public Function WF_FormulaCount() As Long
    Dim ws As Worksheet
    Dim formulaCells As Range
    For Each ws In ThisWorkbook.Worksheets
        Set formulaCells = Nothing
        On Error Resume Next
        Set formulaCells = ws.UsedRange.SpecialCells(xlCellTypeFormulas)
        On Error GoTo 0
        If Not formulaCells Is Nothing Then WF_FormulaCount = WF_FormulaCount + formulaCells.CountLarge
    Next ws
End Function

Public Sub WF_RefreshCharts()
    Dim ws As Worksheet
    Dim chartObject As ChartObject
    For Each ws In ThisWorkbook.Worksheets
        For Each chartObject In ws.ChartObjects
            chartObject.Chart.Refresh
        Next chartObject
    Next ws
End Sub

Public Sub WF_ConfigureDepthChart(ByVal SheetName As String, ByVal ChartIndex As Long, ByVal XTitle As String, ByVal DepthTitle As String)
    Dim profileChart As Chart
    Set profileChart = ThisWorkbook.Worksheets(SheetName).ChartObjects(ChartIndex).Chart
    With profileChart
        .ChartType = xlXYScatterLinesNoMarkers
        .HasAxis(xlCategory, xlPrimary) = True
        .HasAxis(xlValue, xlPrimary) = True
        .Axes(xlCategory).HasTitle = True
        .Axes(xlCategory).AxisTitle.Text = XTitle
        .Axes(xlCategory).TickLabelPosition = xlHigh
        .Axes(xlValue).HasTitle = True
        .Axes(xlValue).AxisTitle.Text = DepthTitle
        .Axes(xlValue).ReversePlotOrder = True
        .Axes(xlCategory).CrossesAt = .Axes(xlValue).MinimumScale
    End With
End Sub

Public Sub WF_ConfigureChartAxes(ByVal SheetName As String, ByVal ChartIndex As Long, ByVal XTitle As String, ByVal YTitle As String)
    Dim targetChart As Chart
    Set targetChart = ThisWorkbook.Worksheets(SheetName).ChartObjects(ChartIndex).Chart
    With targetChart
        .HasAxis(xlCategory, xlPrimary) = True
        .HasAxis(xlValue, xlPrimary) = True
        .Axes(xlCategory).HasTitle = (Len(XTitle) > 0)
        If Len(XTitle) > 0 Then .Axes(xlCategory).AxisTitle.Text = XTitle
        .Axes(xlValue).HasTitle = (Len(YTitle) > 0)
        If Len(YTitle) > 0 Then .Axes(xlValue).AxisTitle.Text = YTitle
    End With
End Sub

Public Sub WF_SetChartXAxisNumberFormat(ByVal SheetName As String, ByVal ChartIndex As Long, ByVal NumberFormatCode As String)
    With ThisWorkbook.Worksheets(SheetName).ChartObjects(ChartIndex).Chart
        .Axes(xlCategory).TickLabels.NumberFormat = NumberFormatCode
    End With
End Sub

Public Sub WellForge_UnitSwitchSelfTest()
    Dim wsUnits As Worksheet
    Dim oldSystem As Variant, oldCustom As Variant
    Dim model As String, domainName As String, valueSheet As String, valueAddress As String, labelAddress As String
    Dim siValue As Double, imperialValue As Double, customValue As Double
    Dim siLabel As String, imperialLabel As String, customLabel As String
    Dim failureNumber As Long, failureDescription As String

    Set wsUnits = ThisWorkbook.Worksheets("Unit Map")
    oldSystem = wsUnits.Range("B5").Value2
    oldCustom = wsUnits.Range("J8:J40").Value2
    model = WF_ModelKind()
    Select Case model
        Case "API7G": domainName = "Force": valueSheet = "Results": valueAddress = "B6": labelAddress = "B4"
        Case "HYDRAULICS": domainName = "Pressure": valueSheet = "Results": valueAddress = "B6": labelAddress = "C6"
        Case "TORQUE_DRAG": domainName = "Length": valueSheet = "Results": valueAddress = "A7": labelAddress = "A5"
        Case "BHA": domainName = "Stress": valueSheet = "Results": valueAddress = "C6": labelAddress = "C5"
        Case "DIRECTIONAL": domainName = "Length": valueSheet = "Summary": valueAddress = "B6": labelAddress = "C6"
        Case Else: Err.Raise vbObjectError + 8210, "WellForge_UnitSwitchSelfTest", "Unknown workbook model"
    End Select

    On Error GoTo Failed
    wsUnits.Range("B5").Value2 = "SI"
    WF_UpdateUnitMap: WF_DispatchModel model
    siValue = CDbl(ThisWorkbook.Worksheets(valueSheet).Range(valueAddress).Value2)
    siLabel = CStr(ThisWorkbook.Worksheets(valueSheet).Range(labelAddress).Value2)
    WF_AssertUnitSwitch siValue, siLabel, WF_UnitLabel(domainName), "SI"

    wsUnits.Range("B5").Value2 = "Imperial"
    WF_UpdateUnitMap: WF_DispatchModel model
    imperialValue = CDbl(ThisWorkbook.Worksheets(valueSheet).Range(valueAddress).Value2)
    imperialLabel = CStr(ThisWorkbook.Worksheets(valueSheet).Range(labelAddress).Value2)
    WF_AssertUnitSwitch imperialValue, imperialLabel, WF_UnitLabel(domainName), "Imperial"
    If StrComp(siLabel, imperialLabel, vbTextCompare) = 0 Then Err.Raise vbObjectError + 8211, "WellForge_UnitSwitchSelfTest", "Unit label did not change from SI to Imperial"
    If Abs(siValue) > 0.000000001 And Abs(siValue - imperialValue) <= Abs(siValue) * 0.000000001 Then Err.Raise vbObjectError + 8212, "WellForge_UnitSwitchSelfTest", "Displayed value did not change from SI to Imperial"

    wsUnits.Range("J8:J40").Value2 = "SI"
    wsUnits.Cells(WF_UnitRow(domainName), 10).Value2 = "Imperial"
    wsUnits.Range("B5").Value2 = "Custom"
    WF_UpdateUnitMap: WF_DispatchModel model
    customValue = CDbl(ThisWorkbook.Worksheets(valueSheet).Range(valueAddress).Value2)
    customLabel = CStr(ThisWorkbook.Worksheets(valueSheet).Range(labelAddress).Value2)
    WF_AssertUnitSwitch customValue, customLabel, WF_UnitLabel(domainName), "Custom"
    If StrComp(customLabel, imperialLabel, vbTextCompare) <> 0 Then Err.Raise vbObjectError + 8213, "WellForge_UnitSwitchSelfTest", "Custom domain label did not preserve its Imperial selection"
    If Abs(customValue - imperialValue) > Application.Max(0.000000001, Abs(imperialValue) * 0.000000001) Then Err.Raise vbObjectError + 8214, "WellForge_UnitSwitchSelfTest", "Custom domain value did not preserve its Imperial selection"
    WF_AssertModelDepthCharts model

Cleanup:
    On Error Resume Next
    wsUnits.Range("J8:J40").Value2 = oldCustom
    wsUnits.Range("B5").Value2 = oldSystem
    WF_UpdateUnitMap
    WF_DispatchModel model
    WF_RefreshCharts
    On Error GoTo 0
    If failureNumber <> 0 Then Err.Raise failureNumber, "WellForge_UnitSwitchSelfTest", failureDescription
    Exit Sub
Failed:
    failureNumber = Err.Number: failureDescription = Err.Description
    Resume Cleanup
End Sub

Private Sub WF_AssertUnitSwitch(ByVal DisplayValue As Double, ByVal DisplayLabel As String, ByVal ExpectedUnit As String, ByVal ModeName As String)
    If Not IsNumeric(DisplayValue) Then Err.Raise vbObjectError + 8215, "WF_AssertUnitSwitch", ModeName & " display value is not numeric"
    If InStr(1, DisplayLabel, ExpectedUnit, vbTextCompare) = 0 Then Err.Raise vbObjectError + 8216, "WF_AssertUnitSwitch", ModeName & " display label does not contain " & ExpectedUnit
End Sub

Private Sub WF_AssertModelDepthCharts(ByVal model As String)
    Dim chartIndex As Long, operationName As Variant
    Select Case model
        Case "HYDRAULICS"
            For chartIndex = 1 To 3: WF_AssertDepthChart "Hydraulics Charts", chartIndex: Next chartIndex
            For chartIndex = 1 To 3: WF_AssertDepthChart "Hydraulics Dashboard", chartIndex: Next chartIndex
        Case "TORQUE_DRAG"
            For chartIndex = 1 To 3: WF_AssertDepthChart "Graphs", chartIndex: Next chartIndex
            For Each operationName In Array("PUW", "SOW", "BKR", "SLD", "ROT", "DRLG")
                WF_AssertDepthChart CStr(operationName), 1: WF_AssertDepthChart CStr(operationName), 2
            Next operationName
            For chartIndex = 1 To 12: WF_AssertDepthChart "Operation Charts", chartIndex: Next chartIndex
            For chartIndex = 1 To 4: WF_AssertDepthChart "Engineering Dashboard", chartIndex: Next chartIndex
        Case "DIRECTIONAL"
            For chartIndex = 2 To 6: WF_AssertDepthChart "Graphs", chartIndex: Next chartIndex
    End Select
End Sub

Private Sub WF_AssertDepthChart(ByVal SheetName As String, ByVal ChartIndex As Long)
    Dim profileChart As Chart
    Set profileChart = ThisWorkbook.Worksheets(SheetName).ChartObjects(ChartIndex).Chart
    With profileChart
        If .ChartType <> xlXYScatterLinesNoMarkers Then Err.Raise vbObjectError + 8217, "WF_AssertDepthChart", SheetName & " chart " & CStr(ChartIndex) & " is not an XY depth roadmap"
        If .Axes(xlValue).ReversePlotOrder <> True Then Err.Raise vbObjectError + 8218, "WF_AssertDepthChart", SheetName & " chart " & CStr(ChartIndex) & " does not reverse depth"
        If .Axes(xlCategory).TickLabelPosition <> xlHigh Then Err.Raise vbObjectError + 8219, "WF_AssertDepthChart", SheetName & " chart " & CStr(ChartIndex) & " does not place the response axis at the top"
    End With
End Sub

Public Sub WF_WriteEngineStatus(ByVal StateText As String, ByVal DetailText As String)
    Dim ws As Worksheet
    If Not WF_SheetExists("Summary") Then Exit Sub
    Set ws = ThisWorkbook.Worksheets("Summary")
    ws.Range("J3:N3").Merge
    ws.Range("J3").Value2 = "Calculation client / engine"
    ws.Range("J4:K7").Value2 = Empty
    ws.Range("J4:J7").Value2 = Application.Transpose(Array("State", "Engine", "Calculated time", "Detail"))
    ws.Range("K4").Value2 = StateText
    ws.Range("K5").Value2 = WF_ENGINE_VERSION
    ws.Range("K6").Value2 = Format$(Now, "yyyy-mm-dd hh:nn:ss") & " local"
    ws.Range("K7").Value2 = DetailText
    ws.Range("K4:K7").NumberFormat = "@"
End Sub

Private Sub WF_ReplacePocLanguage()
    Dim ws As Worksheet
    For Each ws In ThisWorkbook.Worksheets
        On Error Resume Next
        ws.UsedRange.Replace What:="Formula-driven planning and review workbook", Replacement:="compiled calculation client/engine workbook", LookAt:=xlPart, MatchCase:=False
        ws.UsedRange.Replace What:="No VBA and no external links.", Replacement:="Compiled calculation client/engine; no external links.", LookAt:=xlPart, MatchCase:=False
        ws.UsedRange.Replace What:="External links / VBA", Replacement:="External links / compiled client", LookAt:=xlPart, MatchCase:=False
        On Error GoTo 0
    Next ws
End Sub

Private Sub WF_InstallControls()
    Dim ws As Worksheet
    Set ws = ThisWorkbook.Worksheets("Summary")
    WF_AddButton ws, "WF_Calculate", "Calculate", "WellForge_CalculateAll", ws.Range("J10").Left, ws.Range("J10").Top, 95, 24
    WF_AddButton ws, "WF_Validate", "Validate", "WellForge_ValidateModel", ws.Range("L10").Left, ws.Range("L10").Top, 95, 24
    WF_AddButton ws, "WF_LoadJson", "Load JSON", "WellForge_LoadJson", ws.Range("J12").Left, ws.Range("J12").Top, 95, 24
    WF_AddButton ws, "WF_SaveJson", "Save JSON", "WellForge_SaveJson", ws.Range("L12").Left, ws.Range("L12").Top, 95, 24
End Sub

Private Sub WF_AddButton(ByVal ws As Worksheet, ByVal ButtonName As String, ByVal Caption As String, ByVal MacroName As String, ByVal LeftPos As Double, ByVal TopPos As Double, ByVal Width As Double, ByVal Height As Double)
    Dim button As Button
    On Error Resume Next
    ws.Buttons(ButtonName).Delete
    On Error GoTo 0
    Set button = ws.Buttons.Add(LeftPos, TopPos, Width, Height)
    button.Name = ButtonName
    button.Caption = Caption
    button.OnAction = MacroName
End Sub

Public Sub WF_ClearRange(ByVal SheetName As String, ByVal Address As String)
    ThisWorkbook.Worksheets(SheetName).Range(Address).ClearContents
End Sub

Public Sub WF_StatusCell(ByVal target As Range, ByVal StateText As String)
    target.Value2 = StateText
    target.Font.Bold = True
    Select Case UCase$(StateText)
        Case "PASS", "CLEAR", "READY", "OK", "WITHIN SCREENING LIMIT": target.Interior.Color = RGB(204, 251, 241)
        Case "REVIEW", "CAUTION", "ENGINEERING REVIEW": target.Interior.Color = RGB(254, 243, 199)
        Case "STOP", "ERROR", "INVALID": target.Interior.Color = RGB(254, 226, 226)
        Case Else: target.Interior.Color = RGB(243, 244, 246)
    End Select
End Sub
