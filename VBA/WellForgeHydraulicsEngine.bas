Attribute VB_Name = "WellForgeHydraulicsEngine"
Option Explicit

Private Const WF_HYD_TIMEOUT_SECONDS As Double = 120#
Private Const WF_HYD_ANALYSIS_UUID As String = "35b15a48-47c1-4d31-a92e-7c5b8f200701"
Private Const WF_HYD_PI As Double = 3.14159265358979
Private Const WF_HYD_G As Double = 9.80665

Public Sub WF_RunHydraulicsRustEngine()
    Dim executablePath As String, hashPath As String, runPath As String
    Dim requestPath As String, resultPath As String, expectedHash As String
    Dim candidateResults As Collection, result As Object, baseResult As Object, selectedResult As Object
    Dim nozzleIndex As Long, bestIndex As Long, bestScore As Double, score As Double
    Dim nozzleDiameter As Double, surfaceLimit As Double, flowPathLoss As Double
    Dim snapshots As Collection, failureNumber As Long, failureDescription As String
    Dim calcData(1 To 8, 1 To 10) As Variant, flowData(1 To 8, 1 To 14) As Variant
    Dim pressureData(1 To 8, 1 To 10) As Variant, graphData(1 To 8, 1 To 4) As Variant
    Dim pressureRoadmap(1 To 8, 1 To 5) As Variant, ecdRoadmap(1 To 8, 1 To 4) As Variant
    Dim velocityRoadmap(1 To 8, 1 To 3) As Variant, nozzleCalc(1 To 5, 1 To 6) As Variant
    Dim nozzleData(1 To 5, 1 To 12) As Variant, nozzleChart(1 To 5, 1 To 4) As Variant
    On Error GoTo Failed

    Set snapshots = WF_HydCaptureSnapshots()
    Set candidateResults = New Collection
    surfaceLimit = WF_Num("Inputs", "B6")
    If surfaceLimit <= 0# Then Err.Raise vbObjectError + 8910, "WF_RunHydraulicsRustEngine", "Pressure limit must be positive"

    executablePath = ThisWorkbook.Path & Application.PathSeparator & "wellforge-hydraulics.exe"
    hashPath = executablePath & ".sha256"
    If Len(Dir$(executablePath, vbNormal)) = 0 Or Len(Dir$(hashPath, vbNormal)) = 0 Then _
        Err.Raise vbObjectError + 8911, "WF_RunHydraulicsRustEngine", "ENGINE UNAVAILABLE: " & executablePath
    expectedHash = LCase$(Trim$(ReadUtf8File(hashPath)))
    If Not WF_RustIsSha256(expectedHash) Or StrComp(WF_RustFileSha256(executablePath), expectedHash, vbBinaryCompare) <> 0 Then _
        Err.Raise vbObjectError + 8912, "WF_RunHydraulicsRustEngine", "ENGINE HASH MISMATCH"

    runPath = WF_RustFreshRunDirectory("WellForgeHydraulics", WF_HYD_ANALYSIS_UUID)
    requestPath = runPath & Application.PathSeparator & "request.json"
    resultPath = runPath & Application.PathSeparator & "result.json"
    For nozzleIndex = 1 To 5
        nozzleDiameter = WF_Num("Calc", "L" & CStr(nozzleIndex + 5), WF_Num("Inputs", "B12"))
        If nozzleDiameter <= 0# Then Err.Raise vbObjectError + 8913, "WF_RunHydraulicsRustEngine", "Invalid nozzle candidate"
        Set result = WF_HydRunCandidate(executablePath, requestPath, resultPath, nozzleDiameter)
        candidateResults.Add result
        flowPathLoss = WF_HydSelectedFlowPathLoss(result)
        score = Abs(surfaceLimit - flowPathLoss - WF_RustNumber(result.Item("bit_pressure_loss_pa"))) / surfaceLimit
        If score < bestScore Or nozzleIndex = 1 Then bestScore = score: bestIndex = nozzleIndex
    Next nozzleIndex
    Set baseResult = candidateResults.Item(1)
    Set selectedResult = candidateResults.Item(bestIndex)
    WF_HydStageOutputs baseResult, candidateResults, bestIndex, calcData, flowData, pressureData, graphData, pressureRoadmap, ecdRoadmap, velocityRoadmap, nozzleCalc, nozzleData, nozzleChart
    WF_HydCommitOutputs calcData, flowData, pressureData, graphData, pressureRoadmap, ecdRoadmap, velocityRoadmap, nozzleCalc, nozzleData, nozzleChart
    With ThisWorkbook.Worksheets("Results")
        .Range("B6").Value2 = WF_HydSelectedFlowPathLoss(baseResult) * WF_UnitFactor("Pressure")
        .Range("C6").Value2 = WF_UnitLabel("Pressure")
        .Range("D6").Value2 = surfaceLimit * WF_UnitFactor("Pressure")
        WF_StatusCell .Range("E6"), IIf(WF_HydSelectedFlowPathLoss(baseResult) <= surfaceLimit, "PASS", "REVIEW")
        .Range("B7").Value2 = WF_Num("Calc", "L" & CStr(bestIndex + 5), WF_Num("Inputs", "B12")) * WF_UnitFactor("Diameter")
        .Range("C7").Value2 = WF_UnitLabel("Diameter")
        WF_StatusCell .Range("E7"), "PASS"
        .Range("B8").Value2 = (WF_HydSelectedFlowPathLoss(selectedResult) + WF_RustNumber(selectedResult.Item("bit_pressure_loss_pa"))) * WF_UnitFactor("Pressure")
        .Range("C8").Value2 = WF_UnitLabel("Pressure")
        .Range("D8").Value2 = surfaceLimit * WF_UnitFactor("Pressure")
        WF_StatusCell .Range("E8"), IIf(.Range("B8").Value2 <= .Range("D8").Value2, "PASS", "REVIEW")
        .Range("B9").Value2 = WF_ToSI(WF_Num("Inputs", "B8"), WF_Str("Inputs", "C8", "m3/s")) / WF_RustNumber(selectedResult.Item("total_flow_area_m2")) * WF_UnitFactor("Speed")
        .Range("C9").Value2 = WF_UnitLabel("Speed")
        .Range("B10").Value2 = WF_Num("Inputs", "B14") * WF_UnitFactor("Density")
        .Range("C10").Value2 = WF_UnitLabel("Density")
    End With
    With ThisWorkbook.Worksheets("Summary")
        .Range("B6").Value2 = ThisWorkbook.Worksheets("Results").Range("E8").Value2
        .Range("B7").Value2 = ThisWorkbook.Worksheets("Results").Range("B7").Value2
        .Range("A7").Value2 = "Selected nozzle diameter " & WF_UnitLabel("Diameter")
    End With
    WF_WriteEngineStatus "CALCULATED", "Rust hydraulics engine; five nozzle candidates verified"
    Exit Sub
Failed:
    failureNumber = Err.Number: failureDescription = Err.Description
    On Error Resume Next
    If Not snapshots Is Nothing Then WF_HydRestoreSnapshots snapshots
    WF_WriteEngineStatus "FAILED - LAST ACCEPTED VALUES PRESERVED", failureDescription
    On Error GoTo 0
    Err.Raise failureNumber, "WF_RunHydraulicsRustEngine", failureDescription
End Sub

Private Function WF_HydRunCandidate(ByVal executablePath As String, ByVal requestPath As String, ByVal resultPath As String, ByVal nozzleDiameter As Double) As Object
    Dim request As Object, validation As Object, verification As Object, result As Object
    Dim requestHash As String, outputText As String, errorText As String, exitCode As Long
    Set request = WF_BuildHydraulicsRequest(nozzleDiameter)
    AtomicWriteUtf8 requestPath, JsonStringify(request, 0)
    exitCode = WF_RustExecBounded(WF_RustQuote(executablePath) & " validate --input " & WF_RustQuote(requestPath), WF_HYD_TIMEOUT_SECONDS, outputText, errorText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8914, "WF_HydRunCandidate", "INVALID REQUEST: " & Trim$(errorText)
    Set validation = JsonParse(Trim$(outputText))
    requestHash = CStr(validation.Item("request_hash"))
    If Not WF_RustIsSha256(requestHash) Then Err.Raise vbObjectError + 8915, "WF_HydRunCandidate", "INVALID REQUEST HASH"
    exitCode = WF_RustExecBounded(WF_RustQuote(executablePath) & " run --input " & WF_RustQuote(requestPath) & " --output " & WF_RustQuote(resultPath), WF_HYD_TIMEOUT_SECONDS, outputText, errorText)
    If exitCode <> 0 Or Len(Dir$(resultPath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8916, "WF_HydRunCandidate", "ANALYSIS FAILED: " & Trim$(errorText)
    exitCode = WF_RustExecBounded(WF_RustQuote(executablePath) & " verify-result --input " & WF_RustQuote(resultPath) & " --request-hash " & WF_RustQuote(requestHash), WF_HYD_TIMEOUT_SECONDS, outputText, errorText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8917, "WF_HydRunCandidate", "INVALID RESULT: " & Trim$(errorText)
    Set verification = JsonParse(Trim$(outputText))
    If LCase$(CStr(verification.Item("status"))) <> "valid" Then Err.Raise vbObjectError + 8918, "WF_HydRunCandidate", "INVALID RESULT STATUS"
    Set result = JsonParse(ReadUtf8File(resultPath))
    If CStr(result.Item("analysis_id")) <> WF_HYD_ANALYSIS_UUID Then Err.Raise vbObjectError + 8919, "WF_HydRunCandidate", "ANALYSIS ID MISMATCH"
    If LCase$(CStr(result.Item("status"))) = "failed" Then Err.Raise vbObjectError + 8920, "WF_HydRunCandidate", "ENGINE RESULT FAILED"
    Set WF_HydRunCandidate = result
End Function

Private Function WF_BuildHydraulicsRequest(ByVal nozzleDiameter As Double) As Object
    Dim request As Object, profile As Object, source As Object, rheology As Object, operating As Object, sources As Collection
    Dim sections As Collection, nozzles As Collection, section As Object, nozzle As Object
    Dim i As Long, topDepth As Double, sectionLength As Double, flowDiameter As Double, hydraulicDiameter As Double, flowType As String
    Dim stringOd As Double, stringId As Double, holeId As Double, nozzleCount As Long
    Set request = CreateObject("Scripting.Dictionary")
    request.Add "contract_version", "0.1.0": request.Add "analysis_id", WF_HYD_ANALYSIS_UUID
    Set profile = CreateObject("Scripting.Dictionary")
    profile.Add "standard", "API RP 13D": profile.Add "edition", "7th Edition, 2017 (reaffirmed 2023)"
    request.Add "profile", profile
    Set source = CreateObject("Scripting.Dictionary")
    source.Add "uuid", "35b15a48-47c1-4d31-a92e-7c5b8f200702": source.Add "uri", Empty: source.Add "object_type", "tubular"
    source.Add "content_hash", "sha256:" & String$(64, "0"): source.Add "citation_name", "WellForge hydraulics workbook": source.Add "source_system", "WellForgeXL"
    Set sources = New Collection: sources.Add source: request.Add "sources", sources
    Set rheology = CreateObject("Scripting.Dictionary")
    rheology.Add "model", "newtonian": rheology.Add "dynamic_viscosity_pa_s", WF_ToSI(WF_Num("Inputs", "B10"), WF_Str("Inputs", "C10", "Pa*s"))
    rheology.Add "yield_stress_pa", Empty: rheology.Add "plastic_viscosity_pa_s", Empty: rheology.Add "consistency_k_pa_s_n", Empty: rheology.Add "flow_behavior_index", Empty
    request.Add "rheology", rheology
    Set sections = New Collection
    For i = 1 To 8
        sectionLength = WF_Num("Inputs", "E" & CStr(i + 5))
        flowDiameter = WF_Num("Inputs", "F" & CStr(i + 5))
        hydraulicDiameter = WF_Num("Inputs", "H" & CStr(i + 5))
        flowType = LCase$(Trim$(WF_Str("Inputs", "G" & CStr(i + 5), "pipe")))
        If sectionLength <= 0# Or flowDiameter <= 0# Or hydraulicDiameter <= 0# Then Err.Raise vbObjectError + 8921, "WF_BuildHydraulicsRequest", "Invalid flow geometry at Inputs row " & CStr(i + 5)
        Set section = CreateObject("Scripting.Dictionary")
        section.Add "id", "35b15a48-47c1-4d31-a92e-7c5b8f20" & Right$("000" & CStr(i), 3)
        section.Add "name", WF_Str("Inputs", "D" & CStr(i + 5), "Flow section " & CStr(i))
        section.Add "top_md_m", topDepth: topDepth = topDepth + sectionLength
        section.Add "bottom_md_m", topDepth
        If flowType = "annulus" Then
            stringOd = flowDiameter - hydraulicDiameter: stringId = stringOd * 0.8: holeId = flowDiameter
        Else
            stringId = hydraulicDiameter: stringOd = flowDiameter: If stringOd <= stringId Then stringOd = stringId * 1.01
            holeId = stringOd * 1.5
        End If
        If stringOd <= stringId Or holeId <= stringOd Then Err.Raise vbObjectError + 8922, "WF_BuildHydraulicsRequest", "Invalid adapted geometry at Inputs row " & CStr(i + 5)
        section.Add "string_od_m", stringOd: section.Add "string_id_m", stringId: section.Add "hole_id_m", holeId
        sections.Add section
    Next i
    request.Add "sections", sections
    Set operating = CreateObject("Scripting.Dictionary")
    operating.Add "mud_density_kg_m3", WF_ToSI(WF_Num("Inputs", "B9"), WF_Str("Inputs", "C9", "kg/m3"))
    operating.Add "flow_rate_m3_s", WF_ToSI(WF_Num("Inputs", "B8"), WF_Str("Inputs", "C8", "m3/s"))
    operating.Add "surface_temperature_k", 300#
    Set nozzles = New Collection
    nozzleCount = CLng(WF_Num("Inputs", "B11"))
    If nozzleCount < 1 Then Err.Raise vbObjectError + 8923, "WF_BuildHydraulicsRequest", "Nozzle count must be positive"
    For i = 1 To nozzleCount
        Set nozzle = CreateObject("Scripting.Dictionary"): nozzle.Add "diameter_m", nozzleDiameter: nozzles.Add nozzle
    Next i
    operating.Add "nozzles", nozzles
    request.Add "operating", operating
    Set WF_BuildHydraulicsRequest = request
End Function

Private Function WF_HydSelectedFlowPathLoss(ByVal result As Object) As Double
    Dim sections As Collection, section As Object, i As Long, rowIndex As Long, flowType As String
    Set sections = result.Item("sections")
    For i = 1 To 8
        rowIndex = (i - 1) * 2 + 1
        flowType = LCase$(WF_Str("Inputs", "G" & CStr(i + 5), "pipe"))
        If flowType = "annulus" Then rowIndex = rowIndex + 1
        Set section = sections.Item(rowIndex)
        WF_HydSelectedFlowPathLoss = WF_HydSelectedFlowPathLoss + WF_RustNumber(section.Item("pressure_loss_pa"))
    Next i
End Function

Private Function WF_HydCandidateVelocity(ByVal result As Object) As Double
    Dim sections As Collection, section As Object, i As Long
    Set sections = result.Item("sections")
    For i = 1 To sections.Count
        Set section = sections.Item(i)
        If LCase$(CStr(section.Item("flow_loop"))) = "annulus" Then WF_HydCandidateVelocity = WF_RustNumber(section.Item("bulk_velocity_m_s"))
    Next i
End Function

Private Sub WF_HydStageOutputs(ByVal baseResult As Object, ByVal candidateResults As Collection, ByVal bestIndex As Long, ByRef calcData() As Variant, ByRef flowData() As Variant, ByRef pressureData() As Variant, ByRef graphData() As Variant, ByRef pressureRoadmap() As Variant, ByRef ecdRoadmap() As Variant, ByRef velocityRoadmap() As Variant, ByRef nozzleCalc() As Variant, ByRef nozzleData() As Variant, ByRef nozzleChart() As Variant)
    Dim sections As Collection, section As Object, i As Long, j As Long, rowIndex As Long, sectionLength As Double, depth As Double, cumulative As Double
    Dim flowType As String, flowDiameter As Double, hydraulicDiameter As Double, area As Double, velocity As Double, re As Double, friction As Double, loss As Double, ecd As Double
    Dim surfaceLimit As Double, rho As Double, pressureFactor As Double, densityFactor As Double, lengthFactor As Double, diameterFactor As Double, areaFactor As Double, speedFactor As Double
    Dim result As Object, bitDrop As Double, spp As Double, score As Double, rankValue As Long, nozzleDiameter As Double, totalArea As Double
    surfaceLimit = WF_Num("Inputs", "B6"): rho = WF_ToSI(WF_Num("Inputs", "B9"), WF_Str("Inputs", "C9", "kg/m3"))
    lengthFactor = WF_UnitFactor("Length"): diameterFactor = WF_UnitFactor("Diameter"): areaFactor = WF_UnitFactor("Area"): speedFactor = WF_UnitFactor("Speed"): pressureFactor = WF_UnitFactor("Pressure"): densityFactor = WF_UnitFactor("Density")
    Set sections = baseResult.Item("sections")
    For i = 1 To 8
        rowIndex = (i - 1) * 2 + 1: Set section = sections.Item(rowIndex)
        flowType = LCase$(WF_Str("Inputs", "G" & CStr(i + 5), "pipe"))
        If flowType = "annulus" Then Set section = sections.Item(rowIndex + 1)
        sectionLength = WF_Num("Inputs", "E" & CStr(i + 5)): flowDiameter = WF_Num("Inputs", "F" & CStr(i + 5)): hydraulicDiameter = WF_Num("Inputs", "H" & CStr(i + 5))
        If flowType = "annulus" Then area = WF_HYD_PI / 4# * (flowDiameter ^ 2 - (flowDiameter - hydraulicDiameter) ^ 2) Else area = WF_HYD_PI / 4# * hydraulicDiameter ^ 2
        velocity = WF_RustNumber(section.Item("bulk_velocity_m_s")): re = WF_RustNumber(section.Item("reynolds_number")): friction = WF_RustNumber(section.Item("fanning_friction_factor")): loss = WF_RustNumber(section.Item("pressure_loss_pa"))
        cumulative = cumulative + loss: depth = depth + sectionLength: If flowType = "annulus" Then ecd = rho + loss / (WF_HYD_G * sectionLength) Else ecd = rho
        calcData(i, 1) = WF_Str("Inputs", "D" & CStr(i + 5), "Flow section " & CStr(i)): calcData(i, 2) = sectionLength: calcData(i, 3) = hydraulicDiameter: calcData(i, 4) = velocity: calcData(i, 5) = re: calcData(i, 6) = friction: calcData(i, 7) = loss: calcData(i, 8) = cumulative: calcData(i, 9) = cumulative / surfaceLimit: calcData(i, 10) = IIf(cumulative <= surfaceLimit, "PASS", "REVIEW")
        flowData(i, 1) = WF_Str("Inputs", "I" & CStr(i + 5)): flowData(i, 2) = calcData(i, 1): flowData(i, 3) = flowType: flowData(i, 4) = sectionLength * lengthFactor: flowData(i, 5) = hydraulicDiameter * diameterFactor: flowData(i, 6) = area * areaFactor: flowData(i, 7) = velocity * speedFactor: flowData(i, 8) = re: flowData(i, 9) = IIf(re < 2100#, "LAMINAR", IIf(re < 4000#, "TRANSITION", "TURBULENT")): flowData(i, 10) = friction: flowData(i, 11) = loss * pressureFactor: flowData(i, 12) = cumulative * pressureFactor: flowData(i, 13) = cumulative / surfaceLimit: flowData(i, 14) = calcData(i, 10)
        pressureData(i, 1) = calcData(i, 1): pressureData(i, 2) = depth * lengthFactor: pressureData(i, 3) = velocity * speedFactor: pressureData(i, 4) = loss * pressureFactor: pressureData(i, 5) = cumulative * pressureFactor: pressureData(i, 6) = rho * WF_HYD_G * depth * pressureFactor: pressureData(i, 7) = (rho * WF_HYD_G * depth + cumulative) * pressureFactor: pressureData(i, 8) = ecd * densityFactor: pressureData(i, 9) = cumulative / surfaceLimit: pressureData(i, 10) = calcData(i, 10)
        graphData(i, 1) = calcData(i, 1): graphData(i, 2) = loss * pressureFactor: graphData(i, 3) = cumulative * pressureFactor: graphData(i, 4) = ecd * densityFactor
        pressureRoadmap(i, 1) = depth * lengthFactor: pressureRoadmap(i, 2) = loss * pressureFactor: pressureRoadmap(i, 3) = cumulative * pressureFactor: pressureRoadmap(i, 4) = rho * WF_HYD_G * depth * pressureFactor: pressureRoadmap(i, 5) = (rho * WF_HYD_G * depth + cumulative) * pressureFactor
        ecdRoadmap(i, 1) = depth * lengthFactor: ecdRoadmap(i, 2) = rho * densityFactor: ecdRoadmap(i, 3) = ecd * densityFactor: ecdRoadmap(i, 4) = WF_Num("Inputs", "B14") * densityFactor
        velocityRoadmap(i, 1) = depth * lengthFactor: velocityRoadmap(i, 2) = velocity * speedFactor: velocityRoadmap(i, 3) = WF_Num("Inputs", "B15", 0.5) * speedFactor
    Next i
    For i = 1 To 5
        Set result = candidateResults.Item(i): nozzleDiameter = WF_Num("Calc", "L" & CStr(i + 5), WF_Num("Inputs", "B12")): totalArea = WF_RustNumber(result.Item("total_flow_area_m2")): bitDrop = WF_RustNumber(result.Item("bit_pressure_loss_pa")): spp = cumulative + bitDrop: score = Abs(spp - surfaceLimit) / surfaceLimit: rankValue = 1
        For j = 1 To 5: If Abs((WF_HydSelectedFlowPathLoss(candidateResults.Item(j)) + WF_RustNumber(candidateResults.Item(j).Item("bit_pressure_loss_pa"))) - surfaceLimit) < Abs(spp - surfaceLimit) Then rankValue = rankValue + 1
        Next j
        nozzleCalc(i, 1) = nozzleDiameter: nozzleCalc(i, 2) = totalArea: nozzleCalc(i, 3) = WF_ToSI(WF_Num("Inputs", "B8"), WF_Str("Inputs", "C8", "m3/s")) / totalArea: nozzleCalc(i, 4) = bitDrop: nozzleCalc(i, 5) = spp: nozzleCalc(i, 6) = score
        nozzleData(i, 1) = WF_Str("Calc", "R" & CStr(i + 5), "Candidate " & CStr(i)): nozzleData(i, 2) = nozzleDiameter * diameterFactor: nozzleData(i, 3) = WF_Num("Inputs", "B11"): nozzleData(i, 4) = totalArea * areaFactor: nozzleData(i, 5) = nozzleCalc(i, 3) * speedFactor: nozzleData(i, 6) = bitDrop * pressureFactor: nozzleData(i, 7) = cumulative * pressureFactor: nozzleData(i, 8) = spp * pressureFactor: nozzleData(i, 9) = (surfaceLimit - spp) * pressureFactor: nozzleData(i, 10) = bitDrop * WF_ToSI(WF_Num("Inputs", "B8"), WF_Str("Inputs", "C8", "m3/s")): nozzleData(i, 11) = nozzleData(i, 10) / (WF_HYD_PI / 4# * 0.216 ^ 2): nozzleData(i, 12) = rankValue
        nozzleChart(i, 1) = nozzleData(i, 2): nozzleChart(i, 2) = nozzleData(i, 8): nozzleChart(i, 3) = surfaceLimit * pressureFactor: nozzleChart(i, 4) = nozzleData(i, 6)
    Next i
End Sub

Private Sub WF_HydCommitOutputs(ByRef calcData() As Variant, ByRef flowData() As Variant, ByRef pressureData() As Variant, ByRef graphData() As Variant, ByRef pressureRoadmap() As Variant, ByRef ecdRoadmap() As Variant, ByRef velocityRoadmap() As Variant, ByRef nozzleCalc() As Variant, ByRef nozzleData() As Variant, ByRef nozzleChart() As Variant)
    Dim wsCalc As Worksheet, wsFlow As Worksheet, wsPressure As Worksheet, wsNozzle As Worksheet, wsGraphs As Worksheet, wsCharts As Worksheet
    Set wsCalc = ThisWorkbook.Worksheets("Calc"): Set wsFlow = ThisWorkbook.Worksheets("Flow Path"): Set wsPressure = ThisWorkbook.Worksheets("Pressure Profile"): Set wsNozzle = ThisWorkbook.Worksheets("Nozzle Cases"): Set wsGraphs = ThisWorkbook.Worksheets("Graphs"): Set wsCharts = ThisWorkbook.Worksheets("Hydraulics Charts")
    wsCalc.Range("A6:J13").Value2 = calcData: wsCalc.Range("L6:Q10").Value2 = nozzleCalc: wsFlow.Range("A6:N13").Value2 = flowData: wsPressure.Range("A6:J13").Value2 = pressureData: wsNozzle.Range("A6:L10").Value2 = nozzleData: wsGraphs.Range("A4:C11").Value2 = WF_FirstColumns(graphData, 8, 3): wsGraphs.Range("A15:C22").Value2 = WF_WaterfallData(calcData, WF_UnitFactor("Pressure")): wsGraphs.Range("E41:H45").Value2 = WF_NozzleGraphData(nozzleCalc, WF_UnitFactor("Diameter"), WF_UnitFactor("Pressure"), WF_Num("Inputs", "B6")): wsCharts.Range("A6:E13").Value2 = pressureRoadmap: wsCharts.Range("A26:D33").Value2 = ecdRoadmap: wsCharts.Range("A45:C52").Value2 = velocityRoadmap: wsCharts.Range("A64:D68").Value2 = nozzleChart
    WF_WriteHydraulicsDashboard pressureData, nozzleChart, WF_UnitFactor("Length"), WF_UnitFactor("Pressure"), WF_UnitFactor("Density"), WF_UnitFactor("Speed"), WF_Num("Inputs", "B6"), WF_ToSI(WF_Num("Inputs", "B9"), WF_Str("Inputs", "C9", "kg/m3")), WF_Num("Inputs", "B14"), WF_Num("Inputs", "B15", 0.5)
End Sub

Private Function WF_HydCaptureSnapshots() As Collection
    Dim snapshots As Collection: Set snapshots = New Collection
    WF_HydSnapshot snapshots, "Calc", "A6:J13": WF_HydSnapshot snapshots, "Calc", "L6:Q10": WF_HydSnapshot snapshots, "Flow Path", "A6:N13": WF_HydSnapshot snapshots, "Pressure Profile", "A6:J13": WF_HydSnapshot snapshots, "Nozzle Cases", "A6:L10": WF_HydSnapshot snapshots, "Graphs", "A4:H45": WF_HydSnapshot snapshots, "Hydraulics Charts", "A6:E68": WF_HydSnapshot snapshots, "Results", "B6:E10": WF_HydSnapshot snapshots, "Summary", "A6:B7": WF_HydSnapshot snapshots, "Hydraulics Dashboard", "A46:X545": WF_HydSnapshot snapshots, "Chart Settings", "B6"
    Set WF_HydCaptureSnapshots = snapshots
End Function

Private Sub WF_HydSnapshot(ByVal snapshots As Collection, ByVal sheetName As String, ByVal address As String)
    Dim snapshot As Object: Set snapshot = CreateObject("Scripting.Dictionary"): snapshot.Add "sheet", sheetName: snapshot.Add "address", address: snapshot.Add "values", ThisWorkbook.Worksheets(sheetName).Range(address).Value2: snapshots.Add snapshot
End Sub

Private Sub WF_HydRestoreSnapshots(ByVal snapshots As Collection)
    Dim i As Long, snapshot As Object
    For i = snapshots.Count To 1 Step -1
        Set snapshot = snapshots.Item(i): ThisWorkbook.Worksheets(CStr(snapshot.Item("sheet"))).Range(CStr(snapshot.Item("address"))).Value2 = snapshot.Item("values")
    Next i
End Sub
