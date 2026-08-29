Attribute VB_Name = "WellForgeBha"
Option Explicit

Private Const BHA_PI As Double = 3.14159265358979

Public Sub WF_CalcBHA()
    Dim wsIn As Worksheet, wsCalc As Worksheet, wsResults As Worksheet, wsSummary As Worksheet, wsGraphs As Worksheet
    Dim wsAssembly As Worksheet, wsModes As Worksheet, wsBending As Worksheet, wsGeometry As Worksheet, wsTendency As Worksheet, wsPolar As Worksheet
    Dim calcData(1 To 6, 1 To 14) As Variant, resultData(1 To 6, 1 To 6) As Variant
    Dim assemblyData(1 To 6, 1 To 13) As Variant, modeData(1 To 18, 1 To 11) As Variant, bendingData(1 To 6, 1 To 12) As Variant
    Dim tendencyData(1 To 12, 1 To 8) As Variant, roseData(1 To 12, 1 To 7) As Variant, polarData(1 To 13, 1 To 5) As Variant
    Dim ringData(1 To 12, 1 To 5) As Variant, graphModes(1 To 6, 1 To 3) As Variant, graphStress(1 To 6, 1 To 2) As Variant
    Dim graphRose(1 To 13, 1 To 4) As Variant, graphTendency(1 To 6, 1 To 3) As Variant
    Dim geometryData(1 To 30, 1 To 15) As Variant, geometryProfile(1 To 30, 1 To 5) As Variant
    Dim campbellData(1 To 10, 1 To 10) As Variant, modalProfile(1 To 6, 1 To 4) As Variant
    Dim componentDeflection(1 To 6) As Double, componentMoment(1 To 6) As Double, componentStress(1 To 6) As Double
    Dim componentTop(1 To 6) As Double, componentBottom(1 To 6) As Double
    Dim r As Long, componentIndex As Long, modeIndex As Long, outputIndex As Long
    Dim rpm As Double, youngPa As Double, steelDensity As Double, wob1 As Double, wob2 As Double
    Dim componentLength As Double, od As Double, insideDiameter As Double, support As Double
    Dim area As Double, inertia As Double, mass As Double, stiffness As Double, frequency As Double
    Dim moment As Double, stress As Double, tendency1 As Double, tendency2 As Double
    Dim avgTendency1 As Double, avgTendency2 As Double, angle As Double, magnitude1 As Double, magnitude2 As Double, maxMagnitude As Double
    Dim minFrequency As Double, maxStress As Double, reviewCount As Long, topPosition As Double
    Dim stressFactor As Double, lengthFactor As Double, diameterFactor As Double, forceFactor As Double, angleFactor As Double
    Dim separation As Double, ratio As Double, curvature As Double, strain As Double, deflection As Double
    Dim holeDiameter As Double, displayScale As Double, projectionSign As Double, localFraction As Double
    Dim distanceFromBit As Double, centreline As Double, radialClearance As Double, geometryIndex As Long, sampleIndex As Long
    Dim torqueFactor As Double, rpmIndex As Long

    Set wsIn = ThisWorkbook.Worksheets("Inputs"): Set wsCalc = ThisWorkbook.Worksheets("Calc")
    Set wsResults = ThisWorkbook.Worksheets("Results"): Set wsSummary = ThisWorkbook.Worksheets("Summary")
    Set wsGraphs = ThisWorkbook.Worksheets("Graphs"): Set wsAssembly = ThisWorkbook.Worksheets("BHA Assembly")
    Set wsModes = ThisWorkbook.Worksheets("Vibration Modes"): Set wsBending = ThisWorkbook.Worksheets("Bending Response")
    Set wsGeometry = ThisWorkbook.Worksheets("BHA Geometry View")
    Set wsTendency = ThisWorkbook.Worksheets("Tendency Matrix"): Set wsPolar = ThisWorkbook.Worksheets("Polar Plot")

    rpm = WF_Num("Inputs", "B5")
    youngPa = WF_ToSI(WF_Num("Inputs", "B7"), WF_Str("Inputs", "C7", "GPa"))
    steelDensity = WF_Num("Inputs", "B8")
    wob1 = WF_Num("Inputs", "B9")
    wob2 = WF_Num("Inputs", "B10")
    holeDiameter = WF_Num("Inputs", "B11")
    projectionSign = IIf(UCase$(WF_Str("Inputs", "B12", "Highside")) = "LOWSIDE", -1#, 1#)
    displayScale = WF_Num("Inputs", "B13")
    If youngPa <= 0# Or steelDensity <= 0# Or wob1 < 0# Or wob2 < 0# Or holeDiameter <= 0# Or displayScale <= 0# Then Err.Raise vbObjectError + 8600, "WF_CalcBHA", "Invalid material, WOB or projection inputs"
    stressFactor = WF_UnitFactor("Stress"): lengthFactor = WF_UnitFactor("Length")
    diameterFactor = WF_UnitFactor("Diameter"): forceFactor = WF_UnitFactor("Force"): angleFactor = WF_UnitFactor("Angle")
    torqueFactor = WF_UnitFactor("Torque")
    minFrequency = 1E+99

    For r = 1 To 6
        componentLength = CDbl(wsIn.Cells(r + 5, 5).Value2)
        od = CDbl(wsIn.Cells(r + 5, 6).Value2)
        insideDiameter = CDbl(wsIn.Cells(r + 5, 7).Value2)
        support = CDbl(wsIn.Cells(r + 5, 8).Value2)
        If componentLength <= 0# Or od <= insideDiameter Or insideDiameter < 0# Or support <= 0# Then Err.Raise vbObjectError + 8601, "WF_CalcBHA", "Invalid BHA geometry at Inputs row " & CStr(r + 5)
        area = BHA_PI / 4# * (od ^ 2 - insideDiameter ^ 2)
        inertia = BHA_PI / 64# * (od ^ 4 - insideDiameter ^ 4)
        mass = area * componentLength * steelDensity
        stiffness = 48# * youngPa * inertia * support / componentLength ^ 3
        frequency = 1# / (2# * BHA_PI) * Sqr(stiffness / mass)
        moment = wob1 * componentLength * support / 4#
        stress = moment * od / (2# * inertia)
        tendency1 = wob1 / stiffness * 0.0001
        tendency2 = wob2 / stiffness * 0.0001

        calcData(r, 1) = wsIn.Cells(r + 5, 4).Value2: calcData(r, 2) = componentLength: calcData(r, 3) = od: calcData(r, 4) = insideDiameter
        calcData(r, 5) = area: calcData(r, 6) = inertia: calcData(r, 7) = mass: calcData(r, 8) = stiffness
        calcData(r, 9) = frequency: calcData(r, 10) = moment: calcData(r, 11) = stress: calcData(r, 12) = tendency1: calcData(r, 13) = tendency2
        calcData(r, 14) = IIf(frequency > rpm / 60# * 1.2, "CLEAR", "REVIEW")
        If calcData(r, 14) = "REVIEW" Then reviewCount = reviewCount + 1

        resultData(r, 1) = calcData(r, 1): resultData(r, 2) = frequency: resultData(r, 3) = stress * stressFactor
        resultData(r, 4) = tendency1: resultData(r, 5) = tendency2: resultData(r, 6) = wsIn.Cells(r + 5, 9).Value2

        componentTop(r) = topPosition
        assemblyData(r, 1) = resultData(r, 6): assemblyData(r, 2) = calcData(r, 1): assemblyData(r, 3) = topPosition * lengthFactor
        topPosition = topPosition + componentLength
        componentBottom(r) = topPosition
        assemblyData(r, 4) = topPosition * lengthFactor: assemblyData(r, 5) = componentLength * lengthFactor
        assemblyData(r, 6) = od * diameterFactor: assemblyData(r, 7) = insideDiameter * diameterFactor
        assemblyData(r, 8) = area * WF_UnitFactor("Area"): assemblyData(r, 9) = inertia
        assemblyData(r, 10) = mass: assemblyData(r, 11) = support
        assemblyData(r, 12) = IIf(r = 1, "Bit / formation interface", "BHA component"): assemblyData(r, 13) = "PASS"

        curvature = moment / (youngPa * inertia): strain = curvature * od / 2#: deflection = moment * componentLength ^ 2 / (8# * youngPa * inertia)
        componentDeflection(r) = deflection: componentMoment(r) = moment: componentStress(r) = stress
        bendingData(r, 1) = calcData(r, 1): bendingData(r, 2) = componentLength * lengthFactor
        bendingData(r, 3) = moment * WF_UnitFactor("Torque"): bendingData(r, 4) = curvature / lengthFactor
        bendingData(r, 5) = strain: bendingData(r, 6) = stress * stressFactor: bendingData(r, 7) = deflection * lengthFactor
        bendingData(r, 8) = tendency1: bendingData(r, 9) = tendency2: bendingData(r, 10) = support
        bendingData(r, 11) = IIf(stress > 350000000#, "REVIEW", "PASS"): bendingData(r, 12) = resultData(r, 6)

        graphModes(r, 1) = ((componentTop(r) + componentBottom(r)) / 2#) * lengthFactor: graphModes(r, 2) = frequency: graphModes(r, 3) = stress * stressFactor
        graphStress(r, 1) = graphModes(r, 1): graphStress(r, 2) = stress * stressFactor
        graphTendency(r, 1) = calcData(r, 1): graphTendency(r, 2) = tendency1: graphTendency(r, 3) = tendency2
        avgTendency1 = avgTendency1 + tendency1 / 6#: avgTendency2 = avgTendency2 + tendency2 / 6#
        If frequency < minFrequency Then minFrequency = frequency
        If stress > maxStress Then maxStress = stress
    Next r

    ' Projection is an explicit geometric interference indication only; it is not solved contact or reaction force.
    geometryIndex = 0
    For componentIndex = 1 To 6
        For sampleIndex = 0 To 4
            geometryIndex = geometryIndex + 1
            localFraction = sampleIndex / 4#
            distanceFromBit = componentTop(componentIndex) + (componentBottom(componentIndex) - componentTop(componentIndex)) * localFraction
            centreline = projectionSign * componentDeflection(componentIndex) * displayScale * Sin(BHA_PI * localFraction)
            radialClearance = holeDiameter / 2# - Abs(centreline) - CDbl(calcData(componentIndex, 3)) / 2#
            geometryData(geometryIndex, 1) = distanceFromBit
            geometryData(geometryIndex, 2) = distanceFromBit * lengthFactor
            geometryData(geometryIndex, 3) = calcData(componentIndex, 1)
            geometryData(geometryIndex, 4) = localFraction
            geometryData(geometryIndex, 5) = centreline
            geometryData(geometryIndex, 6) = centreline * diameterFactor
            geometryData(geometryIndex, 7) = holeDiameter / 2# * diameterFactor
            geometryData(geometryIndex, 8) = -holeDiameter / 2# * diameterFactor
            geometryData(geometryIndex, 9) = (centreline + CDbl(calcData(componentIndex, 3)) / 2#) * diameterFactor
            geometryData(geometryIndex, 10) = (centreline - CDbl(calcData(componentIndex, 3)) / 2#) * diameterFactor
            geometryData(geometryIndex, 11) = (centreline + CDbl(calcData(componentIndex, 4)) / 2#) * diameterFactor
            geometryData(geometryIndex, 12) = (centreline - CDbl(calcData(componentIndex, 4)) / 2#) * diameterFactor
            geometryData(geometryIndex, 13) = radialClearance * diameterFactor
            geometryData(geometryIndex, 14) = 0#
            geometryData(geometryIndex, 15) = IIf(radialClearance < 0#, "OVERLAP INDICATION", "CLEARANCE")
            geometryProfile(geometryIndex, 1) = distanceFromBit * lengthFactor
            geometryProfile(geometryIndex, 2) = componentMoment(componentIndex) * torqueFactor
            geometryProfile(geometryIndex, 3) = componentStress(componentIndex) * stressFactor
            geometryProfile(geometryIndex, 4) = 350000000# * stressFactor
            geometryProfile(geometryIndex, 5) = centreline * diameterFactor
        Next sampleIndex
    Next componentIndex

    For rpmIndex = 1 To 10
        campbellData(rpmIndex, 1) = (rpmIndex - 1) * 30#
        campbellData(rpmIndex, 2) = campbellData(rpmIndex, 1) / 60#
        campbellData(rpmIndex, 3) = 3# * campbellData(rpmIndex, 1) / 60#
        campbellData(rpmIndex, 4) = 5# * campbellData(rpmIndex, 1) / 60#
        For componentIndex = 1 To 6
            campbellData(rpmIndex, componentIndex + 4) = calcData(componentIndex, 9)
        Next componentIndex
    Next rpmIndex
    For componentIndex = 1 To 6
        modalProfile(componentIndex, 1) = ((componentTop(componentIndex) + componentBottom(componentIndex)) / 2#) * lengthFactor
        modalProfile(componentIndex, 2) = calcData(componentIndex, 9)
        modalProfile(componentIndex, 3) = 2# * calcData(componentIndex, 9)
        modalProfile(componentIndex, 4) = 3# * calcData(componentIndex, 9)
    Next componentIndex

    outputIndex = 0
    For componentIndex = 1 To 6
        For modeIndex = 1 To 3
            outputIndex = outputIndex + 1
            ratio = (rpm / 60#) / (CDbl(calcData(componentIndex, 9)) * modeIndex)
            separation = Abs(CDbl(calcData(componentIndex, 9)) * modeIndex - rpm / 60#) / CDbl(calcData(componentIndex, 9)) / modeIndex
            modeData(outputIndex, 1) = calcData(componentIndex, 1): modeData(outputIndex, 2) = modeIndex
            modeData(outputIndex, 3) = CDbl(calcData(componentIndex, 9)) * modeIndex: modeData(outputIndex, 4) = rpm / 60#
            modeData(outputIndex, 5) = ratio: modeData(outputIndex, 6) = separation: modeData(outputIndex, 7) = calcData(componentIndex, 7)
            modeData(outputIndex, 8) = calcData(componentIndex, 8): modeData(outputIndex, 9) = 0.03 + 0.01 * (modeIndex - 1)
            modeData(outputIndex, 10) = IIf(separation < 0.2, "REVIEW", "CLEAR"): modeData(outputIndex, 11) = resultData(componentIndex, 6)
        Next modeIndex
    Next componentIndex

    For r = 1 To 12
        angle = (r - 1) * BHA_PI / 6#
        magnitude1 = avgTendency1 * (1# + 0.35 * Cos(angle))
        magnitude2 = avgTendency2 * (1# + 0.35 * Cos(angle))
        tendencyData(r, 1) = (r - 1) * 30#: tendencyData(r, 2) = angle * angleFactor
        tendencyData(r, 3) = wob1 * forceFactor: tendencyData(r, 4) = magnitude1
        tendencyData(r, 5) = wob2 * forceFactor: tendencyData(r, 6) = magnitude2
        tendencyData(r, 7) = WF_Quadrant((r - 1) * 30#): tendencyData(r, 8) = IIf(Application.Max(magnitude1, magnitude2) > 0.05, "REVIEW", "SCREEN")
        roseData(r, 1) = angle * angleFactor: roseData(r, 2) = magnitude1: roseData(r, 3) = magnitude1 * Cos(angle): roseData(r, 4) = magnitude1 * Sin(angle)
        roseData(r, 5) = magnitude2: roseData(r, 6) = magnitude2 * Cos(angle): roseData(r, 7) = magnitude2 * Sin(angle)
        polarData(r, 1) = (r - 1) * 30#: polarData(r, 2) = magnitude1 * Sin(angle): polarData(r, 3) = magnitude1 * Cos(angle)
        polarData(r, 4) = magnitude2 * Sin(angle): polarData(r, 5) = magnitude2 * Cos(angle)
        If magnitude1 > maxMagnitude Then maxMagnitude = magnitude1
        If magnitude2 > maxMagnitude Then maxMagnitude = magnitude2
    Next r
    For r = 1 To 5: resultData(r, 6) = wsIn.Cells(r + 5, 9).Value2: Next r
    For r = 1 To 12
        ringData(r, 1) = (r - 1) * 30#: ringData(r, 2) = maxMagnitude * 0.25: ringData(r, 3) = maxMagnitude * 0.5
        ringData(r, 4) = maxMagnitude * 0.75: ringData(r, 5) = maxMagnitude
    Next r
    For r = 1 To 5: polarData(13, r) = polarData(1, r): Next r
    For r = 1 To 4: graphRose(13, r) = polarData(13, r + 1): Next r
    For r = 1 To 12
        graphRose(r, 1) = roseData(r, 3): graphRose(r, 2) = roseData(r, 4): graphRose(r, 3) = roseData(r, 6): graphRose(r, 4) = roseData(r, 7)
    Next r

    wsCalc.Range("A6:N11").Value2 = calcData
    wsCalc.Range("P6:AD35").Value2 = geometryData
    wsCalc.Range("AE6:AI35").Value2 = geometryProfile
    wsResults.Range("A6:F11").Value2 = resultData
    wsResults.Range("G6:M17").Value2 = roseData
    wsAssembly.Range("A6:M11").Value2 = assemblyData
    wsModes.Range("A6:K23").Value2 = modeData
    wsModes.Range("M6:V15").Value2 = campbellData
    wsModes.Range("X6:AA11").Value2 = modalProfile
    wsBending.Range("A6:L11").Value2 = bendingData
    wsTendency.Range("A6:H17").Value2 = tendencyData
    wsPolar.Range("A6:E18").Value2 = polarData
    wsPolar.Range("H6:L17").Value2 = ringData
    wsGraphs.Range("A4:C9").Value2 = graphModes
    wsGraphs.Range("D4:E9").Value2 = graphStress
    wsGraphs.Range("A21:D33").Value2 = graphRose
    wsGraphs.Range("A56:C61").Value2 = graphTendency

    wsGeometry.Range("B10").Value2 = IIf(projectionSign < 0#, "Lowside", "Highside")
    wsGeometry.Range("B11").Value2 = displayScale
    wsGeometry.Range("B12").Value2 = Application.Min(wsCalc.Range("AB6:AB35"))
    wsGeometry.Range("B13").Value2 = Application.CountIf(wsCalc.Range("AD6:AD35"), "OVERLAP INDICATION")
    wsGeometry.Range("C12").Value2 = WF_UnitLabel("Diameter")
    WF_StatusCell wsGeometry.Range("C13"), IIf(CDbl(wsGeometry.Range("B13").Value2) > 0#, "REVIEW", "CLEAR")
    wsGeometry.Range("D13").Value2 = "Geometric interference indication only; not solved contact or reaction force"

    wsSummary.Range("B6").Value2 = minFrequency
    wsSummary.Range("B7").Value2 = maxStress * stressFactor
    WF_StatusCell wsSummary.Range("B8"), IIf(reviewCount > 0, "REVIEW", "CLEAR")
    wsSummary.Range("A7").Value2 = "Peak bending stress " & WF_UnitLabel("Stress")
    wsResults.Range("C4").Value2 = WF_UnitLabel("Stress")
    wsResults.Range("C5").Value2 = "Bending stress " & WF_UnitLabel("Stress")
    wsResults.Range("G5").Value2 = "Toolface " & WF_UnitLabel("Angle")
    wsAssembly.Range("C5").Value2 = "Top " & WF_UnitLabel("Length")
    wsAssembly.Range("D5").Value2 = "Bottom " & WF_UnitLabel("Length")
    wsAssembly.Range("E5").Value2 = "Length " & WF_UnitLabel("Length")
    wsAssembly.Range("F5").Value2 = "OD " & WF_UnitLabel("Diameter")
    wsAssembly.Range("G5").Value2 = "ID " & WF_UnitLabel("Diameter")
    wsAssembly.Range("H5").Value2 = "Area " & WF_UnitLabel("Area")
    wsBending.Range("B5").Value2 = "Length " & WF_UnitLabel("Length")
    wsBending.Range("C5").Value2 = "Bending moment " & WF_UnitLabel("Torque")
    wsBending.Range("F5").Value2 = "Stress " & WF_UnitLabel("Stress")
    wsBending.Range("G5").Value2 = "Estimated deflection " & WF_UnitLabel("Length")
    wsTendency.Range("B5").Value2 = "Toolface " & WF_UnitLabel("Angle")
    wsTendency.Range("C5").Value2 = "WOB 1 " & WF_UnitLabel("Force")
    wsTendency.Range("E5").Value2 = "WOB 2 " & WF_UnitLabel("Force")
    wsGraphs.Range("C3").Value2 = "Bending stress " & WF_UnitLabel("Stress")
    wsGraphs.Range("E3").Value2 = "Bending stress " & WF_UnitLabel("Stress")
    wsGraphs.Range("A3").Value2 = "Distance from bit " & WF_UnitLabel("Length")
    wsGraphs.Range("D3").Value2 = "Distance from bit " & WF_UnitLabel("Length")
    wsModes.Range("X5").Value2 = "Distance from bit " & WF_UnitLabel("Length")
    WF_ConfigureChartAxes wsGraphs.Name, 1, "Distance from bit (" & WF_UnitLabel("Length") & ")", "Natural frequency (Hz)"
    WF_ConfigureChartAxes wsGraphs.Name, 2, "Distance from bit (" & WF_UnitLabel("Length") & ")", "Bending stress (" & WF_UnitLabel("Stress") & ")"
    WF_ConfigureChartAxes wsGeometry.Name, 1, "Distance from bit (" & WF_UnitLabel("Length") & ")", "Projected lateral position (" & WF_UnitLabel("Diameter") & ")"
    WF_ConfigureChartAxes wsGeometry.Name, 2, "Distance from bit (" & WF_UnitLabel("Length") & ")", "Radial clearance (" & WF_UnitLabel("Diameter") & ")"
    WF_ConfigureChartAxes wsGeometry.Name, 3, "Distance from bit (" & WF_UnitLabel("Length") & ")", "Bending moment (" & WF_UnitLabel("Torque") & ")"
    WF_ConfigureChartAxes wsGeometry.Name, 4, "Distance from bit (" & WF_UnitLabel("Length") & ")", "Bending stress (" & WF_UnitLabel("Stress") & ")"
    WF_ConfigureChartAxes wsModes.Name, 1, "Rotary speed (RPM)", "Frequency (Hz)"
    WF_ConfigureChartAxes wsModes.Name, 2, "Distance from bit (" & WF_UnitLabel("Length") & ")", "Frequency (Hz)"
End Sub

Private Function WF_Quadrant(ByVal Degrees As Double) As String
    If Degrees < 90# Then WF_Quadrant = "BUILD / RIGHT": Exit Function
    If Degrees < 180# Then WF_Quadrant = "DROP / RIGHT": Exit Function
    If Degrees < 270# Then WF_Quadrant = "DROP / LEFT": Exit Function
    WF_Quadrant = "BUILD / LEFT"
End Function
