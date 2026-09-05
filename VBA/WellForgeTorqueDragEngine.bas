Attribute VB_Name = "WellForgeTorqueDragEngine"
Option Explicit

Private Const WF_TD_TIMEOUT_SECONDS As Double = 120#
Private Const WF_TD_ANALYSIS_UUID As String = "2b7f9d1c-44a1-4d31-a92e-7c5b8f200601"
Private Const WF_TD_PI As Double = 3.14159265358979
Private Const WF_TD_G As Double = 9.80665

Public Sub WF_RunTorqueDragRustEngine()
    Dim executablePath As String, hashPath As String, runPath As String, requestPath As String, resultPath As String, expectedHash As String
    Dim operationStates As Variant, operationResults As Collection, i As Long, countRows As Long
    Dim calcData() As Variant, resultData() As Variant, allData() As Variant, snapshots As Collection
    Dim failureNumber As Long, failureDescription As String
    On Error GoTo Failed

    Set snapshots = WF_TDCaptureSnapshots()
    countRows = WF_TDRowCount()
    If countRows < 2 Or countRows > 500 Then Err.Raise vbObjectError + 8930, "WF_RunTorqueDragRustEngine", "Survey must contain between two and 500 stations"
    executablePath = ThisWorkbook.Path & Application.PathSeparator & "wellforge-torque-drag.exe": hashPath = executablePath & ".sha256"
    If Len(Dir$(executablePath, vbNormal)) = 0 Or Len(Dir$(hashPath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8931, "WF_RunTorqueDragRustEngine", "ENGINE UNAVAILABLE: " & executablePath
    expectedHash = LCase$(Trim$(ReadUtf8File(hashPath)))
    If Not WF_RustIsSha256(expectedHash) Or StrComp(WF_RustFileSha256(executablePath), expectedHash, vbBinaryCompare) <> 0 Then Err.Raise vbObjectError + 8932, "WF_RunTorqueDragRustEngine", "ENGINE HASH MISMATCH"
    runPath = WF_RustFreshRunDirectory("WellForgeTorqueDrag", WF_TD_ANALYSIS_UUID): requestPath = runPath & Application.PathSeparator & "request.json": resultPath = runPath & Application.PathSeparator & "result.json"
    operationStates = Array("pickup", "slack_off", "backreaming", "sliding", "rotating_off_bottom", "drilling")
    Set operationResults = New Collection
    For i = LBound(operationStates) To UBound(operationStates)
        operationResults.Add WF_TDRunOperation(executablePath, requestPath, resultPath, CStr(operationStates(i)))
    Next i
    ReDim calcData(1 To countRows, 1 To 14): ReDim resultData(1 To countRows, 1 To 11): ReDim allData(1 To countRows, 1 To 14)
    WF_TDStageOutputs operationResults, countRows, calcData, resultData, allData
    WF_TDCommitOutputs operationResults, countRows, calcData, resultData, allData
    WF_WriteEngineStatus "CALCULATED", "Rust torque-drag engine; six operation states verified"
    Exit Sub
Failed:
    failureNumber = Err.Number: failureDescription = Err.Description
    On Error Resume Next
    If Not snapshots Is Nothing Then WF_TDRestoreSnapshots snapshots
    WF_WriteEngineStatus "FAILED - LAST ACCEPTED VALUES PRESERVED", failureDescription
    On Error GoTo 0
    Err.Raise failureNumber, "WF_RunTorqueDragRustEngine", failureDescription
End Sub

Private Function WF_TDRunOperation(ByVal executablePath As String, ByVal requestPath As String, ByVal resultPath As String, ByVal stateName As String) As Object
    Dim request As Object, validation As Object, verification As Object, result As Object
    Dim requestHash As String, outputText As String, errorText As String, exitCode As Long
    Set request = WF_BuildTorqueDragRequest(stateName): AtomicWriteUtf8 requestPath, JsonStringify(request, 0)
    exitCode = WF_RustExecBounded(WF_RustQuote(executablePath) & " validate --input " & WF_RustQuote(requestPath), WF_TD_TIMEOUT_SECONDS, outputText, errorText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8933, "WF_TDRunOperation", "INVALID REQUEST (" & stateName & "): " & Trim$(errorText)
    Set validation = JsonParse(Trim$(outputText)): requestHash = CStr(validation.Item("request_hash"))
    If Not WF_RustIsSha256(requestHash) Then Err.Raise vbObjectError + 8934, "WF_TDRunOperation", "INVALID REQUEST HASH"
    exitCode = WF_RustExecBounded(WF_RustQuote(executablePath) & " run --input " & WF_RustQuote(requestPath) & " --output " & WF_RustQuote(resultPath), WF_TD_TIMEOUT_SECONDS, outputText, errorText)
    If exitCode <> 0 Or Len(Dir$(resultPath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8935, "WF_TDRunOperation", "ANALYSIS FAILED (" & stateName & "): " & Trim$(errorText)
    exitCode = WF_RustExecBounded(WF_RustQuote(executablePath) & " verify-result --input " & WF_RustQuote(resultPath) & " --request-hash " & WF_RustQuote(requestHash), WF_TD_TIMEOUT_SECONDS, outputText, errorText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8936, "WF_TDRunOperation", "INVALID RESULT (" & stateName & "): " & Trim$(errorText)
    Set verification = JsonParse(Trim$(outputText)): If LCase$(CStr(verification.Item("status"))) <> "valid" Then Err.Raise vbObjectError + 8937, "WF_TDRunOperation", "INVALID RESULT STATUS"
    Set result = JsonParse(ReadUtf8File(resultPath))
    If CStr(result.Item("analysis_id")) <> WF_TD_ANALYSIS_UUID Then Err.Raise vbObjectError + 8938, "WF_TDRunOperation", "ANALYSIS ID MISMATCH"
    If LCase$(CStr(result.Item("status"))) = "failed" Then Err.Raise vbObjectError + 8939, "WF_TDRunOperation", "ENGINE RESULT FAILED"
    Set WF_TDRunOperation = result
End Function

Private Function WF_BuildTorqueDragRequest(ByVal stateName As String) As Object
    Dim request As Object, source As Object, component As Object, spec As Object, operating As Object, sources As Collection, components As Collection, stations As Collection, station As Object
    Dim i As Long, countRows As Long, md As Double, inc As Double, azi As Double, tvd As Double, od As Double, insideDiameter As Double, mudDensity As Double, steelDensity As Double, youngPa As Double, linearWeight As Double, rpm As Double
    Set request = CreateObject("Scripting.Dictionary"): request.Add "contract_version", "0.1.0": request.Add "analysis_id", WF_TD_ANALYSIS_UUID
    Set source = CreateObject("Scripting.Dictionary"): source.Add "uuid", "2b7f9d1c-44a1-4d31-a92e-7c5b8f200602": source.Add "uri", Empty: source.Add "object_type", "tubular": source.Add "content_hash", "sha256:" & String$(64, "0"): source.Add "citation_name", "WellForge torque-drag workbook": source.Add "source_system", "WellForgeXL"
    Set sources = New Collection: sources.Add source: request.Add "sources", sources
    mudDensity = WF_ToSI(WF_Num("Inputs", "B5"), WF_Str("Inputs", "C5", "kg/m3")): od = WF_ToSI(WF_Num("Inputs", "B9"), WF_Str("Inputs", "C9", "m")): insideDiameter = WF_ToSI(WF_Num("Inputs", "B10"), WF_Str("Inputs", "C10", "m")): steelDensity = WF_ToSI(WF_Num("Inputs", "B11", 7850#), WF_Str("Inputs", "C11", "kg/m3")): youngPa = WF_ToSI(WF_Num("Inputs", "B12"), WF_Str("Inputs", "C12", "GPa"))
    If od <= insideDiameter Or insideDiameter < 0# Or mudDensity <= 0# Or steelDensity <= 0# Or youngPa <= 0# Then Err.Raise vbObjectError + 8940, "WF_BuildTorqueDragRequest", "Invalid tubular or material properties"
    linearWeight = steelDensity * WF_TD_PI / 4# * (od ^ 2 - insideDiameter ^ 2)
    Set spec = CreateObject("Scripting.Dictionary"): spec.Add "grade", WF_Str("Inputs", "D9", "S135"): spec.Add "tensile_yield_pa", WF_Num("Inputs", "D10", 931000000#): spec.Add "torsional_yield_nm", WF_Num("Inputs", "D11", 59000#): spec.Add "wear_class_derating", WF_Num("Inputs", "D12", 0.8): spec.Add "safety_factor", WF_Num("Inputs", "D13", 1.1)
    Set component = CreateObject("Scripting.Dictionary"): component.Add "id", "2b7f9d1c-44a1-4d31-a92e-7c5b8f200603": component.Add "name", "Workbook tubular string": component.Add "top_md_m", 0#: component.Add "bottom_md_m", WF_Num("Survey", "A" & CStr(WF_TDRowCount() + 5)): component.Add "od_m", od: component.Add "id_m", insideDiameter: component.Add "linear_weight_kg_m", linearWeight: component.Add "youngs_modulus_pa", youngPa: component.Add "density_kg_m3", steelDensity: component.Add "api7g_spec", spec
    Set components = New Collection: components.Add component: request.Add "components", components
    countRows = WF_TDRowCount(): Set stations = New Collection
    For i = 1 To countRows
        md = WF_Num("Survey", "A" & CStr(i + 5)): inc = WF_ToSI(WF_Num("Survey", "B" & CStr(i + 5)), "rad"): azi = WF_ToSI(WF_Num("Survey", "C" & CStr(i + 5)), "rad"): tvd = WF_Num("Survey", "F" & CStr(i + 5), md * Cos(inc))
        Set station = CreateObject("Scripting.Dictionary"): station.Add "md_m", md: station.Add "inclination_rad", inc: station.Add "azimuth_rad", azi: station.Add "tvd_m", tvd: station.Add "cased", WF_TDIsCased(i + 5): stations.Add station
    Next i
    request.Add "trajectory", stations
    Set operating = CreateObject("Scripting.Dictionary"): operating.Add "state", stateName: operating.Add "weight_on_bit_n", WF_ToSI(WF_Num("Inputs", "B7"), WF_Str("Inputs", "C7", "N")): operating.Add "torque_on_bit_nm", WF_ToSI(WF_Num("Inputs", "B8"), WF_Str("Inputs", "C8", "N-m"))
    rpm = 0#: If stateName = "rotating_off_bottom" Or stateName = "drilling" Or stateName = "backreaming" Then rpm = 1#
    operating.Add "surface_rpm_rad_s", rpm: operating.Add "friction_factor_open_hole", WF_Num("Inputs", "B6"): operating.Add "friction_factor_cased_hole", WF_Num("Inputs", "B6"): operating.Add "mud_density_kg_m3", mudDensity
    request.Add "operating", operating: Set WF_BuildTorqueDragRequest = request
End Function

Private Function WF_TDIsCased(ByVal rowIndex As Long) As Boolean
    Dim value As String: value = LCase$(Trim$(WF_Str("Survey", "G" & CStr(rowIndex), "false"))): WF_TDIsCased = (value = "true" Or value = "yes" Or value = "1" Or value = "cased")
End Function

Private Function WF_TDRowCount() As Long
    Dim i As Long
    For i = 0 To 499
        If Len(Trim$(WF_Str("Survey", "A" & CStr(i + 6)))) = 0 Then Exit For
        WF_TDRowCount = WF_TDRowCount + 1
    Next i
End Function

Private Function WF_TDStation(ByVal result As Object, ByVal index As Long) As Object
    Dim stations As Collection: Set stations = result.Item("stations")
    If index < 1 Or index > stations.Count Then Err.Raise vbObjectError + 8941, "WF_TDStation", "ENGINE STATION COUNT MISMATCH"
    Set WF_TDStation = stations.Item(index)
End Function

Private Function WF_TDBuckling(ByVal result As Object, ByVal index As Long) As Object
    Dim buckling As Collection: Set buckling = result.Item("buckling")
    If index < 1 Or index > buckling.Count Then Err.Raise vbObjectError + 8942, "WF_TDBuckling", "ENGINE BUCKLING COUNT MISMATCH"
    Set WF_TDBuckling = buckling.Item(index)
End Function

Private Sub WF_TDStageOutputs(ByVal operationResults As Collection, ByVal countRows As Long, ByRef calcData() As Variant, ByRef resultData() As Variant, ByRef allData() As Variant)
    Dim r As Long, i As Long, md As Double, prevMd As Double, dmd As Double, dogleg As Double, normalForce As Double, drag As Double, buoyedWeight As Double, peakPooh As Double, minRih As Double, governingDepth As Double
    Dim pick As Object, slack As Object, bkr As Object, sld As Object, rot As Object, drlg As Object, buckle As Object, status As String, lengthFactor As Double, forceFactor As Double, torqueFactor As Double, angleFactor As Double, density As Double, od As Double, insideDiameter As Double, steelDensity As Double
    Dim pickTension As Double, slackTension As Double, sinLimit As Double, helLimit As Double
    lengthFactor = WF_UnitFactor("Length"): forceFactor = WF_UnitFactor("Force"): torqueFactor = WF_UnitFactor("Torque"): angleFactor = WF_UnitFactor("Angle"): density = WF_ToSI(WF_Num("Inputs", "B5"), WF_Str("Inputs", "C5", "kg/m3")): od = WF_ToSI(WF_Num("Inputs", "B9"), WF_Str("Inputs", "C9", "m")): insideDiameter = WF_ToSI(WF_Num("Inputs", "B10"), WF_Str("Inputs", "C10", "m")): steelDensity = WF_ToSI(WF_Num("Inputs", "B11", 7850#), WF_Str("Inputs", "C11", "kg/m3")): buoyedWeight = steelDensity * WF_TD_G * WF_TD_PI / 4# * (od ^ 2 - insideDiameter ^ 2) * (1# - density / steelDensity): minRih = 1E+99
    For r = 1 To countRows
        Set pick = WF_TDStation(operationResults.Item(1), r): Set slack = WF_TDStation(operationResults.Item(2), r): Set bkr = WF_TDStation(operationResults.Item(3), r): Set sld = WF_TDStation(operationResults.Item(4), r): Set rot = WF_TDStation(operationResults.Item(5), r): Set drlg = WF_TDStation(operationResults.Item(6), r): Set buckle = WF_TDBuckling(operationResults.Item(1), r)
        md = WF_RustNumber(pick.Item("md_m")): If r = 1 Then dmd = 0# Else dmd = md - prevMd: If dmd <= 0# Then Err.Raise vbObjectError + 8943, "WF_TDStageOutputs", "ENGINE MD IS NOT INCREASING"
        dogleg = WF_RustNumber(pick.Item("dogleg_rad_m")) * dmd: pickTension = WF_RustNumber(pick.Item("effective_tension_n")): slackTension = WF_RustNumber(slack.Item("effective_tension_n")): normalForce = WF_RustNumber(pick.Item("normal_load_n_m")) * dmd: drag = Abs(pickTension - slackTension) / 2#: sinLimit = WF_RustNumber(buckle.Item("sinusoidal_threshold_n")): helLimit = WF_RustNumber(buckle.Item("helical_threshold_n")): status = IIf(WF_RustNumber(buckle.Item("helical_margin_n")) < 0#, "REVIEW", "PASS")
        calcData(r, 1) = md: calcData(r, 2) = dmd: calcData(r, 3) = dogleg: calcData(r, 4) = buoyedWeight: calcData(r, 5) = normalForce: calcData(r, 6) = drag: calcData(r, 7) = pickTension: calcData(r, 8) = slackTension: calcData(r, 9) = WF_RustNumber(sld.Item("torque_nm")): calcData(r, 10) = WF_RustNumber(rot.Item("torque_nm")): calcData(r, 11) = WF_RustNumber(bkr.Item("torque_nm")): calcData(r, 12) = sinLimit: calcData(r, 13) = helLimit: calcData(r, 14) = status
        resultData(r, 1) = md * lengthFactor: resultData(r, 2) = pickTension * forceFactor: resultData(r, 3) = slackTension * forceFactor: resultData(r, 4) = WF_RustNumber(sld.Item("torque_nm")) * torqueFactor: resultData(r, 5) = WF_RustNumber(rot.Item("torque_nm")) * torqueFactor: resultData(r, 6) = WF_RustNumber(bkr.Item("torque_nm")) * torqueFactor: resultData(r, 7) = sinLimit * forceFactor: resultData(r, 8) = helLimit * forceFactor: resultData(r, 9) = status: resultData(r, 10) = Empty: resultData(r, 11) = WF_Str("Survey", "E" & CStr(r + 5))
        allData(r, 1) = resultData(r, 1): allData(r, 2) = WF_ToSI(WF_Num("Survey", "B" & CStr(r + 5)), "rad") * angleFactor: allData(r, 3) = WF_ToSI(WF_Num("Survey", "C" & CStr(r + 5)), "rad") * angleFactor: allData(r, 4) = md * Cos(WF_ToSI(WF_Num("Survey", "B" & CStr(r + 5)), "rad")) * lengthFactor: allData(r, 5) = resultData(r, 2): allData(r, 6) = resultData(r, 3): allData(r, 7) = resultData(r, 6): allData(r, 8) = resultData(r, 4): allData(r, 9) = resultData(r, 5): allData(r, 10) = WF_RustNumber(drlg.Item("torque_nm")) * torqueFactor: allData(r, 11) = resultData(r, 7): allData(r, 12) = resultData(r, 8): allData(r, 13) = resultData(r, 9): allData(r, 14) = resultData(r, 11)
        If pickTension > peakPooh Then peakPooh = pickTension: If slackTension < minRih Then minRih = slackTension: governingDepth = md
        prevMd = md
    Next r
    For r = 1 To countRows: If CDbl(calcData(r, 8)) = minRih Then resultData(r, 10) = "GOVERNING"
    Next r
End Sub

Private Sub WF_TDCommitOutputs(ByVal operationResults As Collection, ByVal countRows As Long, ByRef calcData() As Variant, ByRef resultData() As Variant, ByRef allData() As Variant)
    Dim wsCalc As Worksheet, wsResults As Worksheet, wsSummary As Worksheet, wsAll As Worksheet, wsGraphs As Worksheet, wsOps As Worksheet, operationNames As Variant, axialColumns As Variant, torqueColumns As Variant, opData() As Variant, opIndex As Long, r As Long, helperRow As Long, peakPooh As Double, minRih As Double, governingDepth As Double
    Set wsCalc = ThisWorkbook.Worksheets("Calc"): Set wsResults = ThisWorkbook.Worksheets("Results"): Set wsSummary = ThisWorkbook.Worksheets("Summary"): Set wsAll = ThisWorkbook.Worksheets("ALL"): Set wsGraphs = ThisWorkbook.Worksheets("Graphs"): Set wsOps = ThisWorkbook.Worksheets("Operation Charts")
    wsCalc.Range("A6:N505").ClearContents: wsResults.Range("A6:K505").ClearContents: wsAll.Range("A6:N505").ClearContents: wsCalc.Range("A6").Resize(countRows, 14).Value2 = calcData: wsResults.Range("A6").Resize(countRows, 11).Value2 = resultData: wsAll.Range("A6").Resize(countRows, 14).Value2 = allData
    minRih = 1E+99
    For r = 1 To countRows
        If CDbl(resultData(r, 2)) > peakPooh Then peakPooh = CDbl(resultData(r, 2))
        If CDbl(resultData(r, 3)) < minRih Then minRih = CDbl(resultData(r, 3)): governingDepth = CDbl(resultData(r, 1))
    Next r
    wsSummary.Range("B6").Value2 = peakPooh: wsSummary.Range("B7").Value2 = minRih: wsSummary.Range("B8").Value2 = governingDepth
    wsSummary.Range("A6").Value2 = "Peak POOH hookload " & WF_UnitLabel("Force"): wsSummary.Range("A7").Value2 = "Lowest RIH axial load " & WF_UnitLabel("Force"): wsSummary.Range("A8").Value2 = "Governing depth " & WF_UnitLabel("Length")
    wsResults.Range("A5").Value2 = "MD " & WF_UnitLabel("Length"): wsResults.Range("B5").Value2 = "POOH " & WF_UnitLabel("Force"): wsResults.Range("C5").Value2 = "RIH " & WF_UnitLabel("Force"): wsResults.Range("D5:F5").Value2 = WF_Row3("Slide torque " & WF_UnitLabel("Torque"), "Rotate torque " & WF_UnitLabel("Torque"), "Backream torque " & WF_UnitLabel("Torque")): wsResults.Range("G5:H5").Value2 = WF_Row2("Sinusoidal limit " & WF_UnitLabel("Force"), "Helical limit " & WF_UnitLabel("Force")): wsAll.Range("A5:L5").Value2 = WF_TDAllHeaders()
    WF_WriteTDGraphs wsGraphs, resultData, countRows
    operationNames = Array("PUW", "SOW", "BKR", "SLD", "ROT", "DRLG"): axialColumns = Array(5, 6, 6, 6, 6, 6): torqueColumns = Array(9, 10, 7, 8, 9, 10): wsOps.Range("A5:E500").ClearContents: helperRow = 5
    For opIndex = 0 To 5
        ReDim opData(1 To countRows, 1 To 8)
        For r = 1 To countRows: opData(r, 1) = allData(r, 1): opData(r, 2) = allData(r, axialColumns(opIndex)): opData(r, 3) = allData(r, torqueColumns(opIndex)): opData(r, 4) = allData(r, 11): opData(r, 5) = allData(r, 12): opData(r, 6) = CDbl(opData(r, 2)) - CDbl(opData(r, 5)): opData(r, 7) = allData(r, 13): opData(r, 8) = allData(r, 14)
        Next r
        With ThisWorkbook.Worksheets(CStr(operationNames(opIndex)))
            .Range("A6:H505").ClearContents
            .Range("A6").Resize(countRows, 8).Value2 = opData
            .Range("A5").Value2 = "MD " & WF_UnitLabel("Length")
            .Range("B5").Value2 = "Axial load " & WF_UnitLabel("Force")
            .Range("C5").Value2 = "Torque " & WF_UnitLabel("Torque")
            .Range("D5:F5").Value2 = WF_Row3("Sinusoidal limit " & WF_UnitLabel("Force"), "Helical limit " & WF_UnitLabel("Force"), "Axial margin " & WF_UnitLabel("Force"))
            .Range("A" & CStr(8 + countRows) & ":G" & CStr(507 + countRows)).ClearContents
            .Range("A" & CStr(8 + countRows) & ":D" & CStr(8 + countRows)).Value2 = WF_Row4("MD " & WF_UnitLabel("Length"), "Axial load " & WF_UnitLabel("Force"), "Sinusoidal limit " & WF_UnitLabel("Force"), "Helical limit " & WF_UnitLabel("Force"))
            .Range("A" & CStr(9 + countRows)).Resize(countRows, 4).Value2 = WF_TDForceHelper(opData, countRows)
            .Range("F" & CStr(8 + countRows) & ":G" & CStr(8 + countRows)).Value2 = WF_Row2("MD " & WF_UnitLabel("Length"), "Torque " & WF_UnitLabel("Torque"))
            .Range("F" & CStr(9 + countRows)).Resize(countRows, 2).Value2 = WF_TDTorqueHelper(opData, countRows)
        End With
        wsOps.Range("A" & CStr(helperRow) & ":E" & CStr(helperRow)).Value2 = WF_Row5("MD " & WF_UnitLabel("Length"), "Axial load " & WF_UnitLabel("Force"), "MD " & WF_UnitLabel("Length"), "Torque " & WF_UnitLabel("Torque"), "Buckling margin " & WF_UnitLabel("Force"))
        wsOps.Range("A" & CStr(helperRow + 1)).Resize(countRows, 5).Value2 = WF_TDOverviewHelper(opData, countRows)
        WF_ConfigureDepthChart CStr(operationNames(opIndex)), 1, "Axial load (" & WF_UnitLabel("Force") & ")", "MD (" & WF_UnitLabel("Length") & ")"
        WF_ConfigureDepthChart CStr(operationNames(opIndex)), 2, "Torque (" & WF_UnitLabel("Torque") & ")", "MD (" & WF_UnitLabel("Length") & ")"
        WF_ConfigureDepthChart wsOps.Name, opIndex * 2 + 1, "Axial load (" & WF_UnitLabel("Force") & ")", "MD (" & WF_UnitLabel("Length") & ")"
        WF_ConfigureDepthChart wsOps.Name, opIndex * 2 + 2, "Torque (" & WF_UnitLabel("Torque") & ")", "MD (" & WF_UnitLabel("Length") & ")"
        helperRow = helperRow + 17
    Next opIndex
    WF_WriteTDIndustryDashboard calcData, resultData, allData, countRows, WF_UnitFactor("Length"), WF_UnitFactor("Force"), WF_UnitFactor("Torque"), WF_UnitFactor("Angle")
End Sub

Private Function WF_TDCaptureSnapshots() As Collection
    Dim snapshots As Collection: Set snapshots = New Collection
    WF_TDSnapshot snapshots, "Calc", "A6:N505": WF_TDSnapshot snapshots, "Results", "A5:K505": WF_TDSnapshot snapshots, "ALL", "A5:N505": WF_TDSnapshot snapshots, "Graphs", "A3:F540": WF_TDSnapshot snapshots, "Operation Charts", "A5:E120": WF_TDSnapshot snapshots, "Summary", "A6:B8": WF_TDSnapshot snapshots, "Hydraulics Dashboard", "A46:X545": WF_TDSnapshot snapshots
    Set WF_TDCaptureSnapshots = snapshots
End Function

Private Sub WF_TDSnapshot(ByVal snapshots As Collection, ByVal sheetName As String, ByVal address As String)
    Dim snapshot As Object: Set snapshot = CreateObject("Scripting.Dictionary"): snapshot.Add "sheet", sheetName: snapshot.Add "address", address: snapshot.Add "values", ThisWorkbook.Worksheets(sheetName).Range(address).Value2: snapshots.Add snapshot
End Sub

Private Sub WF_TDRestoreSnapshots(ByVal snapshots As Collection)
    Dim i As Long, snapshot As Object
    For i = snapshots.Count To 1 Step -1: Set snapshot = snapshots.Item(i): ThisWorkbook.Worksheets(CStr(snapshot.Item("sheet"))).Range(CStr(snapshot.Item("address"))).Value2 = snapshot.Item("values")
    Next i
End Sub
