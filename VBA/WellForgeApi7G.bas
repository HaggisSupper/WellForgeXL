Attribute VB_Name = "WellForgeApi7G"
Option Explicit

Private Const API_PI As Double = 3.14159265358979

Public Sub WF_CalcAPI7G()
    Dim wsIn As Worksheet, wsCalc As Worksheet, wsResults As Worksheet
    Dim wsSummary As Worksheet, wsCatalog As Worksheet, wsCases As Worksheet
    Dim wsDetail As Worksheet, wsGraphs As Worksheet, wsStrength As Worksheet
    Dim calcData(1 To 6, 1 To 10) As Variant
    Dim resultData(1 To 6, 1 To 9) As Variant
    Dim detailData(1 To 36, 1 To 12) As Variant
    Dim graphData(1 To 6, 1 To 3) As Variant
    Dim strengthData(1 To 6, 1 To 4) As Variant
    Dim casePeak(1 To 6, 1 To 3) As Variant
    Dim rowIndex As Long, caseIndex As Long, detailIndex As Long
    Dim od As Double, insideDiameter As Double, area As Double, polarMoment As Double
    Dim density As Double, buoyancy As Double, buoyedLoad As Double
    Dim tensionLimit As Double, operatingTorque As Double, surfaceTorque As Double
    Dim tensionUtil As Double, torqueCapacity As Double, torqueUtil As Double, combinedUtil As Double
    Dim limit As Double, maxCombined As Double, governingSection As String
    Dim forceFactor As Double, torqueFactor As Double
    Dim axialMultiplier As Double, torqueMultiplier As Double, dynamicFactor As Double, combinedFactor As Double
    Dim sectionLoad As Double, caseTorque As Double, caseCombined As Double, peak As Double

    Set wsIn = ThisWorkbook.Worksheets("Inputs")
    Set wsCalc = ThisWorkbook.Worksheets("Calc")
    Set wsResults = ThisWorkbook.Worksheets("Results")
    Set wsSummary = ThisWorkbook.Worksheets("Summary")
    Set wsCatalog = ThisWorkbook.Worksheets("Tubular Catalog")
    Set wsCases = ThisWorkbook.Worksheets("Load Cases")
    Set wsDetail = ThisWorkbook.Worksheets("Section Detail")
    Set wsGraphs = ThisWorkbook.Worksheets("Graphs")
    Set wsStrength = ThisWorkbook.Worksheets("Strength Charts")

    surfaceTorque = WF_Num("Inputs", "K6")
    limit = WF_Num("Inputs", "K8", 0.9)
    forceFactor = WF_UnitFactor("Force")
    torqueFactor = WF_UnitFactor("Torque")

    For rowIndex = 1 To 6
        od = CDbl(wsIn.Cells(rowIndex + 5, 4).Value2)
        insideDiameter = CDbl(wsIn.Cells(rowIndex + 5, 5).Value2)
        If od <= insideDiameter Or insideDiameter < 0# Then Err.Raise vbObjectError + 8300, "WF_CalcAPI7G", "Invalid OD/ID at Inputs row " & CStr(rowIndex + 5)
        density = CDbl(wsIn.Cells(rowIndex + 5, 3).Value2)
        area = API_PI / 4# * (od ^ 2 - insideDiameter ^ 2)
        polarMoment = API_PI / 32# * (od ^ 4 - insideDiameter ^ 4)
        buoyancy = 1# - density / 7850#
        buoyedLoad = CDbl(wsIn.Cells(rowIndex + 5, 6).Value2) * buoyancy
        tensionLimit = CDbl(wsIn.Cells(rowIndex + 5, 7).Value2)
        operatingTorque = CDbl(wsIn.Cells(rowIndex + 5, 8).Value2)
        If tensionLimit <= 0# Or operatingTorque <= 0# Then Err.Raise vbObjectError + 8301, "WF_CalcAPI7G", "Capacity must be positive at Inputs row " & CStr(rowIndex + 5)
        tensionUtil = buoyedLoad / tensionLimit
        torqueCapacity = operatingTorque * 1.35
        torqueUtil = surfaceTorque / torqueCapacity
        combinedUtil = Sqr(tensionUtil ^ 2 + torqueUtil ^ 2)

        calcData(rowIndex, 1) = wsIn.Cells(rowIndex + 5, 1).Value2
        calcData(rowIndex, 2) = area
        calcData(rowIndex, 3) = buoyancy
        calcData(rowIndex, 4) = buoyedLoad
        calcData(rowIndex, 5) = polarMoment
        calcData(rowIndex, 6) = tensionUtil
        calcData(rowIndex, 7) = torqueCapacity
        calcData(rowIndex, 8) = torqueUtil
        calcData(rowIndex, 9) = combinedUtil
        calcData(rowIndex, 10) = IIf(combinedUtil <= limit, "PASS", "REVIEW")

        resultData(rowIndex, 1) = calcData(rowIndex, 1)
        resultData(rowIndex, 2) = buoyedLoad * forceFactor
        resultData(rowIndex, 3) = tensionUtil
        resultData(rowIndex, 4) = torqueUtil
        resultData(rowIndex, 5) = combinedUtil
        resultData(rowIndex, 6) = IIf(tensionUtil <= limit, "PASS", "REVIEW")
        resultData(rowIndex, 7) = IIf(torqueUtil <= limit, "PASS", "REVIEW")
        resultData(rowIndex, 8) = ""
        resultData(rowIndex, 9) = wsIn.Cells(rowIndex + 5, 9).Value2

        graphData(rowIndex, 1) = calcData(rowIndex, 1)
        graphData(rowIndex, 2) = tensionUtil
        graphData(rowIndex, 3) = torqueUtil

        wsCatalog.Cells(rowIndex + 5, 6).Value2 = area
        wsCatalog.Cells(rowIndex + 5, 7).Value2 = polarMoment

        If combinedUtil > maxCombined Then
            maxCombined = combinedUtil
            governingSection = CStr(calcData(rowIndex, 1))
        End If
    Next rowIndex

    For rowIndex = 1 To 6
        If CStr(resultData(rowIndex, 1)) = governingSection Then resultData(rowIndex, 8) = "GOVERNING"
    Next rowIndex

    detailIndex = 0
    For caseIndex = 1 To 6
        axialMultiplier = CDbl(wsCases.Cells(caseIndex + 5, 2).Value2)
        torqueMultiplier = CDbl(wsCases.Cells(caseIndex + 5, 3).Value2)
        dynamicFactor = CDbl(wsCases.Cells(caseIndex + 5, 4).Value2)
        combinedFactor = CDbl(wsCases.Cells(caseIndex + 5, 5).Value2)
        peak = 0#
        For rowIndex = 1 To 6
            detailIndex = detailIndex + 1
            sectionLoad = CDbl(wsIn.Cells(rowIndex + 5, 6).Value2) * axialMultiplier * dynamicFactor
            caseTorque = surfaceTorque * torqueMultiplier * dynamicFactor
            tensionUtil = sectionLoad / CDbl(wsIn.Cells(rowIndex + 5, 7).Value2)
            torqueUtil = caseTorque / CDbl(calcData(rowIndex, 7))
            caseCombined = Sqr(tensionUtil ^ 2 + torqueUtil ^ 2) * combinedFactor
            detailData(detailIndex, 1) = wsCases.Cells(caseIndex + 5, 1).Value2
            detailData(detailIndex, 2) = wsIn.Cells(rowIndex + 5, 1).Value2
            detailData(detailIndex, 3) = sectionLoad * forceFactor
            detailData(detailIndex, 4) = caseTorque * torqueFactor
            detailData(detailIndex, 5) = CDbl(wsIn.Cells(rowIndex + 5, 7).Value2) * forceFactor
            detailData(detailIndex, 6) = CDbl(calcData(rowIndex, 7)) * torqueFactor
            detailData(detailIndex, 7) = tensionUtil
            detailData(detailIndex, 8) = torqueUtil
            detailData(detailIndex, 9) = caseCombined
            detailData(detailIndex, 10) = wsCases.Cells(caseIndex + 5, 6).Value2
            detailData(detailIndex, 11) = IIf(caseCombined <= CDbl(detailData(detailIndex, 10)), "PASS", "REVIEW")
            detailData(detailIndex, 12) = wsIn.Cells(rowIndex + 5, 9).Value2
            If caseCombined > peak Then peak = caseCombined
            If caseIndex = 1 Then strengthData(rowIndex, 2) = caseCombined
            If caseIndex = 4 Then strengthData(rowIndex, 3) = caseCombined
            If caseIndex = 5 Then strengthData(rowIndex, 4) = caseCombined
            strengthData(rowIndex, 1) = wsIn.Cells(rowIndex + 5, 1).Value2
        Next rowIndex
        casePeak(caseIndex, 1) = wsCases.Cells(caseIndex + 5, 1).Value2
        casePeak(caseIndex, 2) = peak
        casePeak(caseIndex, 3) = wsCases.Cells(caseIndex + 5, 6).Value2
    Next caseIndex

    wsCalc.Range("A6:J11").Value2 = calcData
    wsResults.Range("A6:I11").Value2 = resultData
    wsGraphs.Range("A4:C9").Value2 = graphData
    wsGraphs.Range("A23:C28").Value2 = graphData
    wsDetail.Range("A6:L41").Value2 = detailData
    wsStrength.Range("A6:D11").Value2 = strengthData
    wsStrength.Range("A26:C31").Value2 = casePeak

    wsResults.Range("B4").Value2 = WF_UnitLabel("Force")
    wsResults.Range("B5").Value2 = "Buoyed load " & WF_UnitLabel("Force")
    wsDetail.Range("C5").Value2 = "Axial load " & WF_UnitLabel("Force")
    wsDetail.Range("D5").Value2 = "Torque " & WF_UnitLabel("Torque")
    wsDetail.Range("E5").Value2 = "Tension capacity " & WF_UnitLabel("Force")
    wsDetail.Range("F5").Value2 = "Torque capacity " & WF_UnitLabel("Torque")

    wsSummary.Range("B6").Value2 = governingSection
    wsSummary.Range("B7").Value2 = maxCombined
    WF_StatusCell wsSummary.Range("B8"), IIf(maxCombined <= limit, "WITHIN SCREENING LIMIT", "ENGINEERING REVIEW")
End Sub

