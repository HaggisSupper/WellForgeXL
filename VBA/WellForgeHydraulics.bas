Attribute VB_Name = "WellForgeHydraulics"
Option Explicit

Private Const HYD_PI As Double = 3.14159265358979
Private Const HYD_G As Double = 9.80665

Public Sub WF_CalcHydraulics()
    WF_RunHydraulicsRustEngine
End Sub

Private Sub WF_CalcHydraulicsWorksheetModel()
    Dim wsIn As Worksheet, wsCalc As Worksheet, wsResults As Worksheet, wsSummary As Worksheet
    Dim wsFlow As Worksheet, wsNozzle As Worksheet, wsPressure As Worksheet, wsGraphs As Worksheet, wsCharts As Worksheet
    Dim calcData(1 To 8, 1 To 10) As Variant, flowData(1 To 8, 1 To 14) As Variant
    Dim pressureData(1 To 8, 1 To 10) As Variant, graphData(1 To 8, 1 To 4) As Variant
    Dim pressureRoadmap(1 To 8, 1 To 5) As Variant, ecdRoadmap(1 To 8, 1 To 4) As Variant, velocityRoadmap(1 To 8, 1 To 3) As Variant
    Dim nozzleCalc(1 To 5, 1 To 6) As Variant, nozzleData(1 To 5, 1 To 12) As Variant, nozzleChart(1 To 5, 1 To 4) As Variant
    Dim sectionIndex As Long, nozzleIndex As Long, compareIndex As Long
    Dim q As Double, rho As Double, viscosity As Double, surfaceLimit As Double, nozzleCount As Double, dischargeCoefficient As Double
    Dim sectionLength As Double, flowDiameter As Double, hydraulicDiameter As Double, area As Double, velocity As Double
    Dim reynolds As Double, friction As Double, loss As Double, cumulative As Double, depth As Double
    Dim nozzleDiameter As Double, nozzleArea As Double, nozzleVelocity As Double, bitDrop As Double, spp As Double, score As Double
    Dim bestScore As Double, bestIndex As Long, rankValue As Long, ecd As Double, ecdScreen As Double, minimumVelocity As Double
    Dim lengthFactor As Double, diameterFactor As Double, areaFactor As Double, speedFactor As Double
    Dim pressureFactor As Double, densityFactor As Double
    Dim flowType As String, sectionName As String

    Set wsIn = ThisWorkbook.Worksheets("Inputs")
    Set wsCalc = ThisWorkbook.Worksheets("Calc")
    Set wsResults = ThisWorkbook.Worksheets("Results")
    Set wsSummary = ThisWorkbook.Worksheets("Summary")
    Set wsFlow = ThisWorkbook.Worksheets("Flow Path")
    Set wsNozzle = ThisWorkbook.Worksheets("Nozzle Cases")
    Set wsPressure = ThisWorkbook.Worksheets("Pressure Profile")
    Set wsGraphs = ThisWorkbook.Worksheets("Graphs")
    Set wsCharts = ThisWorkbook.Worksheets("Hydraulics Charts")

    surfaceLimit = WF_Num("Inputs", "B6")
    q = WF_Num("Inputs", "B8")
    rho = WF_Num("Inputs", "B9")
    viscosity = WF_Num("Inputs", "B10")
    nozzleCount = WF_Num("Inputs", "B11")
    dischargeCoefficient = WF_Num("Inputs", "B13", 0.95)
    ecdScreen = WF_Num("Inputs", "B14")
    minimumVelocity = WF_Num("Inputs", "B15", 0.5)
    If surfaceLimit <= 0# Or q <= 0# Or rho <= 0# Or viscosity <= 0# Or nozzleCount <= 0# Or dischargeCoefficient <= 0# Then _
        Err.Raise vbObjectError + 8400, "WF_CalcHydraulics", "Pressure limit, flow, density, viscosity, nozzle count and Cd must be positive"

    lengthFactor = WF_UnitFactor("Length")
    diameterFactor = WF_UnitFactor("Diameter")
    areaFactor = WF_UnitFactor("Area")
    speedFactor = WF_UnitFactor("Speed")
    pressureFactor = WF_UnitFactor("Pressure")
    densityFactor = WF_UnitFactor("Density")
    cumulative = 0#: depth = 0#

    For sectionIndex = 1 To 8
        sectionName = CStr(wsIn.Cells(sectionIndex + 5, 4).Value2)
        sectionLength = CDbl(wsIn.Cells(sectionIndex + 5, 5).Value2)
        flowDiameter = CDbl(wsIn.Cells(sectionIndex + 5, 6).Value2)
        flowType = CStr(wsIn.Cells(sectionIndex + 5, 7).Value2)
        hydraulicDiameter = CDbl(wsIn.Cells(sectionIndex + 5, 8).Value2)
        If sectionLength <= 0# Or hydraulicDiameter <= 0# Or flowDiameter <= 0# Then Err.Raise vbObjectError + 8401, "WF_CalcHydraulics", "Invalid flow geometry at Inputs row " & CStr(sectionIndex + 5)

        If StrComp(flowType, "Annulus", vbTextCompare) = 0 Then
            If hydraulicDiameter >= flowDiameter Then Err.Raise vbObjectError + 8402, "WF_CalcHydraulics", "Annular hydraulic diameter must be smaller than hole ID"
            area = HYD_PI / 4# * (flowDiameter ^ 2 - (flowDiameter - hydraulicDiameter) ^ 2)
        Else
            area = HYD_PI / 4# * hydraulicDiameter ^ 2
        End If
        velocity = q / area
        reynolds = rho * velocity * hydraulicDiameter / viscosity
        If reynolds <= 0# Then Err.Raise vbObjectError + 8403, "WF_CalcHydraulics", "Invalid Reynolds number"
        If reynolds > 4000# Then friction = 0.3164 / reynolds ^ 0.25 Else friction = 64# / reynolds
        loss = friction * sectionLength / hydraulicDiameter * (rho * velocity ^ 2 / 2#)
        cumulative = cumulative + loss
        depth = depth + sectionLength
        If StrComp(flowType, "Annulus", vbTextCompare) = 0 Then ecd = rho + loss / (HYD_G * sectionLength) Else ecd = rho

        calcData(sectionIndex, 1) = sectionName
        calcData(sectionIndex, 2) = sectionLength
        calcData(sectionIndex, 3) = hydraulicDiameter
        calcData(sectionIndex, 4) = velocity
        calcData(sectionIndex, 5) = reynolds
        calcData(sectionIndex, 6) = friction
        calcData(sectionIndex, 7) = loss
        calcData(sectionIndex, 8) = cumulative
        calcData(sectionIndex, 9) = cumulative / surfaceLimit
        calcData(sectionIndex, 10) = IIf(cumulative <= surfaceLimit, "PASS", "REVIEW")

        flowData(sectionIndex, 1) = wsIn.Cells(sectionIndex + 5, 9).Value2
        flowData(sectionIndex, 2) = sectionName
        flowData(sectionIndex, 3) = flowType
        flowData(sectionIndex, 4) = sectionLength * lengthFactor
        flowData(sectionIndex, 5) = hydraulicDiameter * diameterFactor
        flowData(sectionIndex, 6) = area * areaFactor
        flowData(sectionIndex, 7) = velocity * speedFactor
        flowData(sectionIndex, 8) = reynolds
        flowData(sectionIndex, 9) = IIf(reynolds < 2100#, "LAMINAR", IIf(reynolds < 4000#, "TRANSITION", "TURBULENT"))
        flowData(sectionIndex, 10) = friction
        flowData(sectionIndex, 11) = loss * pressureFactor
        flowData(sectionIndex, 12) = cumulative * pressureFactor
        flowData(sectionIndex, 13) = cumulative / surfaceLimit
        flowData(sectionIndex, 14) = calcData(sectionIndex, 10)

        pressureData(sectionIndex, 1) = sectionName
        pressureData(sectionIndex, 2) = depth * lengthFactor
        pressureData(sectionIndex, 3) = velocity * speedFactor
        pressureData(sectionIndex, 4) = loss * pressureFactor
        pressureData(sectionIndex, 5) = cumulative * pressureFactor
        pressureData(sectionIndex, 6) = rho * HYD_G * depth * pressureFactor
        pressureData(sectionIndex, 7) = (rho * HYD_G * depth + cumulative) * pressureFactor
        pressureData(sectionIndex, 8) = ecd * densityFactor
        pressureData(sectionIndex, 9) = cumulative / surfaceLimit
        pressureData(sectionIndex, 10) = calcData(sectionIndex, 10)

        graphData(sectionIndex, 1) = sectionName
        graphData(sectionIndex, 2) = loss * pressureFactor
        graphData(sectionIndex, 3) = cumulative * pressureFactor
        graphData(sectionIndex, 4) = ecd * densityFactor

        pressureRoadmap(sectionIndex, 1) = depth * lengthFactor
        pressureRoadmap(sectionIndex, 2) = loss * pressureFactor
        pressureRoadmap(sectionIndex, 3) = cumulative * pressureFactor
        pressureRoadmap(sectionIndex, 4) = rho * HYD_G * depth * pressureFactor
        pressureRoadmap(sectionIndex, 5) = (rho * HYD_G * depth + cumulative) * pressureFactor
        ecdRoadmap(sectionIndex, 1) = depth * lengthFactor
        ecdRoadmap(sectionIndex, 2) = rho * densityFactor
        ecdRoadmap(sectionIndex, 3) = ecd * densityFactor
        ecdRoadmap(sectionIndex, 4) = ecdScreen * densityFactor
        velocityRoadmap(sectionIndex, 1) = depth * lengthFactor
        velocityRoadmap(sectionIndex, 2) = velocity * speedFactor
        velocityRoadmap(sectionIndex, 3) = minimumVelocity * speedFactor
    Next sectionIndex

    bestScore = 1E+99
    For nozzleIndex = 1 To 5
        nozzleDiameter = CDbl(wsCalc.Cells(nozzleIndex + 5, 12).Value2)
        nozzleArea = HYD_PI / 4# * nozzleDiameter ^ 2 * nozzleCount
        nozzleVelocity = q / nozzleArea
        bitDrop = rho / 2# * (nozzleVelocity / dischargeCoefficient) ^ 2
        spp = cumulative + bitDrop
        score = Abs(spp - surfaceLimit) / surfaceLimit
        nozzleCalc(nozzleIndex, 1) = nozzleDiameter
        nozzleCalc(nozzleIndex, 2) = nozzleArea
        nozzleCalc(nozzleIndex, 3) = nozzleVelocity
        nozzleCalc(nozzleIndex, 4) = bitDrop
        nozzleCalc(nozzleIndex, 5) = spp
        nozzleCalc(nozzleIndex, 6) = score
        If score < bestScore Then bestScore = score: bestIndex = nozzleIndex
    Next nozzleIndex

    For nozzleIndex = 1 To 5
        rankValue = 1
        For compareIndex = 1 To 5
            If Abs(surfaceLimit - CDbl(nozzleCalc(compareIndex, 5))) < Abs(surfaceLimit - CDbl(nozzleCalc(nozzleIndex, 5))) Then rankValue = rankValue + 1
        Next compareIndex
        nozzleData(nozzleIndex, 1) = wsCalc.Cells(nozzleIndex + 5, 18).Value2
        nozzleData(nozzleIndex, 2) = CDbl(nozzleCalc(nozzleIndex, 1)) * diameterFactor
        nozzleData(nozzleIndex, 3) = nozzleCount
        nozzleData(nozzleIndex, 4) = CDbl(nozzleCalc(nozzleIndex, 2)) * areaFactor
        nozzleData(nozzleIndex, 5) = CDbl(nozzleCalc(nozzleIndex, 3)) * speedFactor
        nozzleData(nozzleIndex, 6) = CDbl(nozzleCalc(nozzleIndex, 4)) * pressureFactor
        nozzleData(nozzleIndex, 7) = cumulative * pressureFactor
        nozzleData(nozzleIndex, 8) = CDbl(nozzleCalc(nozzleIndex, 5)) * pressureFactor
        nozzleData(nozzleIndex, 9) = (surfaceLimit - CDbl(nozzleCalc(nozzleIndex, 5))) * pressureFactor
        nozzleData(nozzleIndex, 10) = CDbl(nozzleCalc(nozzleIndex, 4)) * q
        nozzleData(nozzleIndex, 11) = nozzleData(nozzleIndex, 10) / (HYD_PI / 4# * 0.216 ^ 2)
        nozzleData(nozzleIndex, 12) = rankValue
        nozzleChart(nozzleIndex, 1) = nozzleData(nozzleIndex, 2)
        nozzleChart(nozzleIndex, 2) = nozzleData(nozzleIndex, 8)
        nozzleChart(nozzleIndex, 3) = surfaceLimit * pressureFactor
        nozzleChart(nozzleIndex, 4) = nozzleData(nozzleIndex, 6)
    Next nozzleIndex

    wsCalc.Range("A6:J13").Value2 = calcData
    wsCalc.Range("L6:Q10").Value2 = nozzleCalc
    wsFlow.Range("A6:N13").Value2 = flowData
    wsPressure.Range("A6:J13").Value2 = pressureData
    wsNozzle.Range("A6:L10").Value2 = nozzleData
    wsGraphs.Range("A4:C11").Value2 = WF_FirstColumns(graphData, 8, 3)
    wsGraphs.Range("A15:C22").Value2 = WF_WaterfallData(calcData, pressureFactor)
    wsGraphs.Range("E41:H45").Value2 = WF_NozzleGraphData(nozzleCalc, diameterFactor, pressureFactor, surfaceLimit)
    wsCharts.Range("A6:E13").Value2 = pressureRoadmap
    wsCharts.Range("A26:D33").Value2 = ecdRoadmap
    wsCharts.Range("A45:C52").Value2 = velocityRoadmap
    wsCharts.Range("A64:D68").Value2 = nozzleChart

    wsResults.Range("B6").Value2 = cumulative * pressureFactor
    wsResults.Range("C6").Value2 = WF_UnitLabel("Pressure")
    wsResults.Range("D6").Value2 = surfaceLimit * pressureFactor
    WF_StatusCell wsResults.Range("E6"), IIf(cumulative <= surfaceLimit, "PASS", "REVIEW")
    wsResults.Range("B7").Value2 = CDbl(nozzleCalc(bestIndex, 1)) * diameterFactor
    wsResults.Range("C7").Value2 = WF_UnitLabel("Diameter")
    WF_StatusCell wsResults.Range("E7"), "PASS"
    wsResults.Range("B8").Value2 = CDbl(nozzleCalc(bestIndex, 5)) * pressureFactor
    wsResults.Range("C8").Value2 = WF_UnitLabel("Pressure")
    wsResults.Range("D8").Value2 = surfaceLimit * pressureFactor
    WF_StatusCell wsResults.Range("E8"), IIf(CDbl(nozzleCalc(bestIndex, 5)) <= surfaceLimit, "PASS", "REVIEW")
    wsResults.Range("B9").Value2 = CDbl(nozzleCalc(bestIndex, 3)) * speedFactor
    wsResults.Range("C9").Value2 = WF_UnitLabel("Speed")
    wsResults.Range("B10").Value2 = WF_Num("Inputs", "B14") * densityFactor
    wsResults.Range("C10").Value2 = WF_UnitLabel("Density")

    wsSummary.Range("B6").Value2 = wsResults.Range("E8").Value2
    wsSummary.Range("B7").Value2 = wsResults.Range("B7").Value2
    wsSummary.Range("A7").Value2 = "Selected nozzle diameter " & WF_UnitLabel("Diameter")

    wsFlow.Range("D5").Value2 = "Length " & WF_UnitLabel("Length")
    wsFlow.Range("E5").Value2 = "Hydraulic dia. " & WF_UnitLabel("Diameter")
    wsFlow.Range("F5").Value2 = "Area " & WF_UnitLabel("Area")
    wsFlow.Range("G5").Value2 = "Velocity " & WF_UnitLabel("Speed")
    wsFlow.Range("K5").Value2 = "Loss " & WF_UnitLabel("Pressure")
    wsFlow.Range("L5").Value2 = "Cumulative " & WF_UnitLabel("Pressure")
    wsPressure.Range("B5").Value2 = "Depth / length " & WF_UnitLabel("Length")
    wsPressure.Range("C5").Value2 = "Velocity " & WF_UnitLabel("Speed")
    wsPressure.Range("D5").Value2 = "Section loss " & WF_UnitLabel("Pressure")
    wsPressure.Range("E5").Value2 = "Cumulative loss " & WF_UnitLabel("Pressure")
    wsPressure.Range("F5").Value2 = "Hydrostatic " & WF_UnitLabel("Pressure")
    wsPressure.Range("G5").Value2 = "Dynamic pressure " & WF_UnitLabel("Pressure")
    wsPressure.Range("H5").Value2 = "ECD " & WF_UnitLabel("Density")
    wsNozzle.Range("B5").Value2 = "Diameter " & WF_UnitLabel("Diameter")
    wsNozzle.Range("D5").Value2 = "Total area " & WF_UnitLabel("Area")
    wsNozzle.Range("E5").Value2 = "Velocity " & WF_UnitLabel("Speed")
    wsNozzle.Range("F5:I5").Value2 = WF_HydRow4("Bit drop " & WF_UnitLabel("Pressure"), "Flow-path loss " & WF_UnitLabel("Pressure"), "SPP " & WF_UnitLabel("Pressure"), "Pressure margin " & WF_UnitLabel("Pressure"))
    wsGraphs.Range("B3:C3").Value2 = WF_HydRow2("Cumulative pressure " & WF_UnitLabel("Pressure"), "Pressure loss " & WF_UnitLabel("Pressure"))
    wsGraphs.Range("B14:C14").Value2 = WF_HydRow2("Base " & WF_UnitLabel("Pressure"), "Increment " & WF_UnitLabel("Pressure"))
    wsGraphs.Range("E40:G40").Value2 = WF_HydRow3("Nozzle diameter " & WF_UnitLabel("Diameter"), "Total SPP " & WF_UnitLabel("Pressure"), "Surface limit " & WF_UnitLabel("Pressure"))
    wsCharts.Range("A5:E5").Value2 = WF_HydRow5("MD " & WF_UnitLabel("Length"), "Section loss " & WF_UnitLabel("Pressure"), "Cumulative loss " & WF_UnitLabel("Pressure"), "Hydrostatic " & WF_UnitLabel("Pressure"), "Dynamic pressure " & WF_UnitLabel("Pressure"))
    wsCharts.Range("A25:D25").Value2 = WF_HydRow4("MD " & WF_UnitLabel("Length"), "Static mud density " & WF_UnitLabel("Density"), "ECD " & WF_UnitLabel("Density"), "ECD screen " & WF_UnitLabel("Density"))
    wsCharts.Range("A44:C44").Value2 = WF_HydRow3("MD " & WF_UnitLabel("Length"), "Flow velocity " & WF_UnitLabel("Speed"), "Minimum annular velocity " & WF_UnitLabel("Speed"))
    wsCharts.Range("A63:D63").Value2 = WF_HydRow4("Nozzle diameter " & WF_UnitLabel("Diameter"), "SPP " & WF_UnitLabel("Pressure"), "Surface limit " & WF_UnitLabel("Pressure"), "Bit drop " & WF_UnitLabel("Pressure"))

    WF_ConfigureDepthChart wsCharts.Name, 1, "Pressure (" & WF_UnitLabel("Pressure") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureDepthChart wsCharts.Name, 2, "Density (" & WF_UnitLabel("Density") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureDepthChart wsCharts.Name, 3, "Velocity (" & WF_UnitLabel("Speed") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureChartAxes wsCharts.Name, 4, "Nozzle diameter (" & WF_UnitLabel("Diameter") & ")", "Pressure (" & WF_UnitLabel("Pressure") & ")"
    WF_ConfigureChartAxes wsGraphs.Name, 3, "Nozzle diameter (" & WF_UnitLabel("Diameter") & ")", "Pressure (" & WF_UnitLabel("Pressure") & ")"
    WF_SetChartXAxisNumberFormat wsCharts.Name, 4, "0.000"
    WF_SetChartXAxisNumberFormat wsGraphs.Name, 3, "0.000"
    WF_WriteHydraulicsDashboard pressureData, nozzleChart, lengthFactor, pressureFactor, densityFactor, speedFactor, surfaceLimit, rho, ecdScreen, minimumVelocity
End Sub

Public Sub WF_WriteHydraulicsDashboard(ByRef pressureData() As Variant, ByRef nozzleChart() As Variant, ByVal lengthFactor As Double, ByVal pressureFactor As Double, ByVal densityFactor As Double, ByVal speedFactor As Double, ByVal surfaceLimit As Double, ByVal rho As Double, ByVal ecdScreen As Double, ByVal minimumVelocity As Double)
    Dim ws As Worksheet, wsSettings As Worksheet
    Dim pressureFamily(1 To 8, 1 To 6) As Variant, ecdFamily(1 To 8, 1 To 6) As Variant, velocityFamily(1 To 8, 1 To 5) As Variant
    Dim r As Long, nearest As Long, selectedMd As Double, lowMultiplier As Double, highMultiplier As Double
    Dim hydrostatic As Double, dynamicPressure As Double, staticDensity As Double, baseEcd As Double, baseVelocity As Double
    Dim reader(1 To 9, 1 To 1) As Variant
    Set ws = ThisWorkbook.Worksheets("Hydraulics Dashboard")
    Set wsSettings = ThisWorkbook.Worksheets("Chart Settings")
    lowMultiplier = WF_Num("Flow Cases", "B6", 0.85)
    highMultiplier = WF_Num("Flow Cases", "B8", 1.15)
    staticDensity = rho * densityFactor

    For r = 1 To 8
        hydrostatic = CDbl(pressureData(r, 6))
        dynamicPressure = CDbl(pressureData(r, 7))
        baseEcd = CDbl(pressureData(r, 8))
        baseVelocity = CDbl(pressureData(r, 3))
        pressureFamily(r, 1) = pressureData(r, 2)
        pressureFamily(r, 2) = hydrostatic + (dynamicPressure - hydrostatic) * lowMultiplier ^ 1.75
        pressureFamily(r, 3) = dynamicPressure
        pressureFamily(r, 4) = hydrostatic + (dynamicPressure - hydrostatic) * highMultiplier ^ 1.75
        pressureFamily(r, 5) = hydrostatic
        pressureFamily(r, 6) = surfaceLimit * pressureFactor
        ecdFamily(r, 1) = pressureData(r, 2)
        ecdFamily(r, 2) = staticDensity + (baseEcd - staticDensity) * lowMultiplier ^ 1.75
        ecdFamily(r, 3) = baseEcd
        ecdFamily(r, 4) = staticDensity + (baseEcd - staticDensity) * highMultiplier ^ 1.75
        ecdFamily(r, 5) = staticDensity
        ecdFamily(r, 6) = ecdScreen * densityFactor
        velocityFamily(r, 1) = pressureData(r, 2)
        velocityFamily(r, 2) = baseVelocity * lowMultiplier
        velocityFamily(r, 3) = baseVelocity
        velocityFamily(r, 4) = baseVelocity * highMultiplier
        velocityFamily(r, 5) = minimumVelocity * speedFactor
    Next r

    ws.Range("A46:X545").ClearContents
    ws.Range("A46").Resize(8, 6).Value2 = pressureFamily
    ws.Range("H46").Resize(8, 6).Value2 = ecdFamily
    ws.Range("O46").Resize(8, 5).Value2 = velocityFamily
    ws.Range("U46").Resize(5, 4).Value2 = nozzleChart
    ws.Range("A45:F45").Value2 = WF_HydDashboardPressureHeaders()
    ws.Range("H45:M45").Value2 = WF_HydDashboardEcdHeaders()
    ws.Range("O45:S45").Value2 = WF_HydDashboardVelocityHeaders()
    ws.Range("U45:X45").Value2 = WF_HydRow4("Nozzle diameter " & WF_UnitLabel("Diameter"), "SPP " & WF_UnitLabel("Pressure"), "Surface limit " & WF_UnitLabel("Pressure"), "Bit drop " & WF_UnitLabel("Pressure"))

    selectedMd = WF_Num("Chart Settings", "B6")
    If selectedMd < CDbl(pressureFamily(1, 1)) / lengthFactor Then selectedMd = CDbl(pressureFamily(1, 1)) / lengthFactor
    If selectedMd > CDbl(pressureFamily(8, 1)) / lengthFactor Then selectedMd = CDbl(pressureFamily(8, 1)) / lengthFactor
    wsSettings.Range("B6").Value2 = selectedMd
    ws.Range("B5").Value2 = selectedMd * lengthFactor: ws.Range("C5").Value2 = WF_UnitLabel("Length")
    nearest = WF_NearestDepthRow(ws.Name, 46, 8, 1, selectedMd * lengthFactor) - 45
    reader(1, 1) = pressureFamily(nearest, 1)
    reader(2, 1) = "Base"
    reader(3, 1) = pressureFamily(nearest, 3)
    reader(4, 1) = pressureFamily(nearest, 6) - pressureFamily(nearest, 3)
    reader(5, 1) = ecdFamily(nearest, 3)
    reader(6, 1) = ecdFamily(nearest, 6) - ecdFamily(nearest, 3)
    reader(7, 1) = velocityFamily(nearest, 3)
    reader(8, 1) = velocityFamily(nearest, 3) - velocityFamily(nearest, 5)
    reader(9, 1) = IIf(CDbl(reader(4, 1)) < 0# Or CDbl(reader(6, 1)) < 0# Or CDbl(reader(8, 1)) < 0#, "REVIEW", "WITHIN LIMITS")
    ws.Range("B8:B16").Value2 = reader

    WF_ConfigureDepthChart ws.Name, 1, "Pressure (" & WF_UnitLabel("Pressure") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureDepthChart ws.Name, 2, "Density (" & WF_UnitLabel("Density") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureDepthChart ws.Name, 3, "Velocity (" & WF_UnitLabel("Speed") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureChartAxes ws.Name, 4, "Nozzle diameter (" & WF_UnitLabel("Diameter") & ")", "Pressure (" & WF_UnitLabel("Pressure") & ")"
    WF_SetChartXAxisNumberFormat ws.Name, 4, "0.000"
End Sub

Private Function WF_HydDashboardPressureHeaders() As Variant
    WF_HydDashboardPressureHeaders = WF_HydRow6("MD " & WF_UnitLabel("Length"), "Low flow pressure " & WF_UnitLabel("Pressure"), "Base flow pressure " & WF_UnitLabel("Pressure"), "High flow pressure " & WF_UnitLabel("Pressure"), "Hydrostatic " & WF_UnitLabel("Pressure"), "Pressure limit " & WF_UnitLabel("Pressure"))
End Function

Private Function WF_HydDashboardEcdHeaders() As Variant
    WF_HydDashboardEcdHeaders = WF_HydRow6("MD " & WF_UnitLabel("Length"), "Low flow ECD " & WF_UnitLabel("Density"), "Base flow ECD " & WF_UnitLabel("Density"), "High flow ECD " & WF_UnitLabel("Density"), "Static density " & WF_UnitLabel("Density"), "ECD limit " & WF_UnitLabel("Density"))
End Function

Private Function WF_HydDashboardVelocityHeaders() As Variant
    WF_HydDashboardVelocityHeaders = WF_HydRow5("MD " & WF_UnitLabel("Length"), "Low flow velocity " & WF_UnitLabel("Speed"), "Base flow velocity " & WF_UnitLabel("Speed"), "High flow velocity " & WF_UnitLabel("Speed"), "Minimum transport velocity " & WF_UnitLabel("Speed"))
End Function

Private Function WF_HydRow6(ByVal a As Variant, ByVal b As Variant, ByVal c As Variant, ByVal d As Variant, ByVal e As Variant, ByVal f As Variant) As Variant
    Dim output(1 To 1, 1 To 6) As Variant: output(1, 1) = a: output(1, 2) = b: output(1, 3) = c: output(1, 4) = d: output(1, 5) = e: output(1, 6) = f: WF_HydRow6 = output
End Function

Public Function WF_FirstColumns(ByRef source() As Variant, ByVal rowCount As Long, ByVal columnCount As Long) As Variant
    Dim result() As Variant, r As Long, c As Long
    ReDim result(1 To rowCount, 1 To columnCount)
    For r = 1 To rowCount: For c = 1 To columnCount: result(r, c) = source(r, c): Next c: Next r
    WF_FirstColumns = result
End Function

Public Function WF_WaterfallData(ByRef calcData() As Variant, ByVal pressureFactor As Double) As Variant
    Dim result(1 To 8, 1 To 3) As Variant, r As Long
    For r = 1 To 8
        result(r, 1) = calcData(r, 1)
        If r = 1 Then result(r, 2) = 0# Else result(r, 2) = CDbl(calcData(r - 1, 8)) * pressureFactor
        result(r, 3) = CDbl(calcData(r, 7)) * pressureFactor
    Next r
    WF_WaterfallData = result
End Function

Public Function WF_NozzleGraphData(ByRef nozzleCalc() As Variant, ByVal diameterFactor As Double, ByVal pressureFactor As Double, ByVal surfaceLimit As Double) As Variant
    Dim result(1 To 5, 1 To 4) As Variant, r As Long
    For r = 1 To 5
        result(r, 1) = CDbl(nozzleCalc(r, 1)) * diameterFactor
        result(r, 2) = CDbl(nozzleCalc(r, 5)) * pressureFactor
        result(r, 3) = surfaceLimit * pressureFactor
        result(r, 4) = nozzleCalc(r, 6)
    Next r
    WF_NozzleGraphData = result
End Function

Private Function WF_HydRow2(ByVal a As Variant, ByVal b As Variant) As Variant
    Dim output(1 To 1, 1 To 2) As Variant: output(1, 1) = a: output(1, 2) = b: WF_HydRow2 = output
End Function

Private Function WF_HydRow3(ByVal a As Variant, ByVal b As Variant, ByVal c As Variant) As Variant
    Dim output(1 To 1, 1 To 3) As Variant: output(1, 1) = a: output(1, 2) = b: output(1, 3) = c: WF_HydRow3 = output
End Function

Private Function WF_HydRow4(ByVal a As Variant, ByVal b As Variant, ByVal c As Variant, ByVal d As Variant) As Variant
    Dim output(1 To 1, 1 To 4) As Variant: output(1, 1) = a: output(1, 2) = b: output(1, 3) = c: output(1, 4) = d: WF_HydRow4 = output
End Function

Private Function WF_HydRow5(ByVal a As Variant, ByVal b As Variant, ByVal c As Variant, ByVal d As Variant, ByVal e As Variant) As Variant
    Dim output(1 To 1, 1 To 5) As Variant: output(1, 1) = a: output(1, 2) = b: output(1, 3) = c: output(1, 4) = d: output(1, 5) = e: WF_HydRow5 = output
End Function

Private Function WF_EcdChartData(ByRef graphData() As Variant) As Variant
    Dim result(1 To 8, 1 To 2) As Variant, r As Long
    For r = 1 To 8: result(r, 1) = graphData(r, 1): result(r, 2) = graphData(r, 4): Next r
    WF_EcdChartData = result
End Function
