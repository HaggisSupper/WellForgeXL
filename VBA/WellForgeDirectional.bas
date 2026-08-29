Attribute VB_Name = "WellForgeDirectional"
Option Explicit

Private Const DD_PI As Double = 3.14159265358979
Private Const DD_TWO_PI As Double = 6.28318530717959

Private pCount As Long, sCount As Long
Private pMD() As Double, pInc() As Double, pAzi() As Double, pTVD() As Double, pNorth() As Double, pEast() As Double, pVS() As Double, pCross() As Double, pDLS() As Double
Private sMD() As Double, sInc() As Double, sAzi() As Double, sTVD() As Double, sNorth() As Double, sEast() As Double, sVS() As Double, sCross() As Double, sDLS() As Double

Public Sub WF_RefreshDirectionalPresentation()
    WF_UpdateDirectionalHeaders
    WF_ConfigureDirectionalCharts
    WF_RefreshCharts
End Sub

Public Sub WF_CalcDirectional()
    Dim wsPlan As Worksheet, wsSurvey As Worksheet, wsCalc As Worksheet, wsResults As Worksheet, wsSummary As Worksheet
    Dim planVisible() As Variant, surveyVisible() As Variant, calcPlan() As Variant, calcSurvey() As Variant, compareData() As Variant
    Dim contractData() As Variant, r As Long, planTvd As Double, planN As Double, planE As Double, coverage As String
    Dim dTvd As Double, dN As Double, dE As Double, dVs As Double, alongError As Double, crossError As Double, horizontalError As Double, error3d As Double
    Dim lengthFactor As Double, gradientFactor As Double, angleFactor As Double, vsa As Double
    Dim latestCross As Double, latest3d As Double, maxDls As Double, overallState As String, nextTarget As String

    Set wsPlan = ThisWorkbook.Worksheets("Plan"): Set wsSurvey = ThisWorkbook.Worksheets("Survey")
    Set wsCalc = ThisWorkbook.Worksheets("Calc"): Set wsResults = ThisWorkbook.Worksheets("Results"): Set wsSummary = ThisWorkbook.Worksheets("Summary")
    lengthFactor = WF_UnitFactor("Length"): gradientFactor = WF_UnitFactor("Angular gradient"): angleFactor = WF_UnitFactor("Angle")
    vsa = WF_DegToRad(WF_Num("Inputs", "B16"))

    WF_CalculatePath wsPlan, WF_Str("Inputs", "E5", "m"), WF_Str("Inputs", "E6", "rad"), vsa, pCount, pMD, pInc, pAzi, pTVD, pNorth, pEast, pVS, pCross, pDLS
    WF_CalculatePath wsSurvey, WF_Str("Inputs", "E7", "m"), WF_Str("Inputs", "E8", "rad"), vsa, sCount, sMD, sInc, sAzi, sTVD, sNorth, sEast, sVS, sCross, sDLS
    If pCount < 2 Or sCount < 2 Then Err.Raise vbObjectError + 8700, "WF_CalcDirectional", "Plan and Survey each require at least two contiguous stations"

    ReDim planVisible(1 To pCount, 1 To 8): ReDim calcPlan(1 To pCount, 1 To 18)
    For r = 1 To pCount
        planVisible(r, 1) = True: planVisible(r, 2) = pTVD(r) * lengthFactor: planVisible(r, 3) = pNorth(r) * lengthFactor
        planVisible(r, 4) = pEast(r) * lengthFactor: planVisible(r, 5) = pVS(r) * lengthFactor: planVisible(r, 6) = pCross(r) * lengthFactor
        planVisible(r, 7) = pDLS(r) * gradientFactor: planVisible(r, 8) = IIf(r = 1 Or pMD(r) > pMD(r - 1), "OK", "INVALID")
        WF_FillCanonicalPathRow calcPlan, r, wsPlan.Cells(r + 6, 1).Value2, pMD, pInc, pAzi, pTVD, pNorth, pEast, pVS, pCross, pDLS
    Next r
    wsPlan.Range("F7:M506").ClearContents: wsPlan.Range("F7").Resize(pCount, 8).Value2 = planVisible
    wsCalc.Range("A7:R506").ClearContents: wsCalc.Range("A7").Resize(pCount, 18).Value2 = calcPlan

    ReDim surveyVisible(1 To sCount, 1 To 20): ReDim calcSurvey(1 To sCount, 1 To 18): ReDim compareData(1 To sCount, 1 To 34): ReDim contractData(1 To sCount, 1 To 13)
    For r = 1 To sCount
        If WF_InterpolatePath(sMD(r), pCount, pMD, pInc, pAzi, pTVD, pNorth, pEast, planTvd, planN, planE, coverage) Then
            dTvd = sTVD(r) - planTvd: dN = sNorth(r) - planN: dE = sEast(r) - planE
            dVs = dN * Cos(vsa) + dE * Sin(vsa): alongError = dVs: crossError = -dN * Sin(vsa) + dE * Cos(vsa)
            horizontalError = Sqr(dN ^ 2 + dE ^ 2): error3d = Sqr(horizontalError ^ 2 + dTvd ^ 2)
            latestCross = crossError: latest3d = error3d
        Else
            dTvd = 0#: dN = 0#: dE = 0#: dVs = 0#: alongError = 0#: crossError = 0#: horizontalError = 0#: error3d = 0#
        End If
        If sDLS(r) > maxDls Then maxDls = sDLS(r)
        surveyVisible(r, 1) = True: surveyVisible(r, 2) = sTVD(r) * lengthFactor: surveyVisible(r, 3) = sNorth(r) * lengthFactor: surveyVisible(r, 4) = sEast(r) * lengthFactor
        surveyVisible(r, 5) = sVS(r) * lengthFactor: surveyVisible(r, 6) = sCross(r) * lengthFactor
        If coverage = "OK" Then
            surveyVisible(r, 7) = planTvd * lengthFactor: surveyVisible(r, 8) = planN * lengthFactor: surveyVisible(r, 9) = planE * lengthFactor
            surveyVisible(r, 10) = dTvd * lengthFactor: surveyVisible(r, 11) = dN * lengthFactor: surveyVisible(r, 12) = dE * lengthFactor
            surveyVisible(r, 13) = dVs * lengthFactor: surveyVisible(r, 14) = alongError * lengthFactor: surveyVisible(r, 15) = crossError * lengthFactor
            surveyVisible(r, 16) = horizontalError * lengthFactor: surveyVisible(r, 17) = error3d * lengthFactor
        End If
        surveyVisible(r, 18) = coverage: surveyVisible(r, 19) = sDLS(r) * gradientFactor: surveyVisible(r, 20) = IIf(r = 1 Or sMD(r) > sMD(r - 1), "OK", "INVALID")
        WF_FillCanonicalPathRow calcSurvey, r, wsSurvey.Cells(r + 6, 1).Value2, sMD, sInc, sAzi, sTVD, sNorth, sEast, sVS, sCross, sDLS
        compareData(r, 1) = coverage: compareData(r, 22) = planTvd: compareData(r, 23) = planN: compareData(r, 24) = planE
        compareData(r, 25) = planN * Cos(vsa) + planE * Sin(vsa): compareData(r, 26) = -planN * Sin(vsa) + planE * Cos(vsa)
        compareData(r, 27) = dTvd: compareData(r, 28) = dN: compareData(r, 29) = dE: compareData(r, 30) = dVs
        compareData(r, 31) = alongError: compareData(r, 32) = crossError: compareData(r, 33) = horizontalError: compareData(r, 34) = error3d
        contractData(r, 1) = wsSurvey.Cells(r + 6, 1).Value2: contractData(r, 2) = sMD(r): contractData(r, 3) = sInc(r): contractData(r, 4) = sAzi(r)
        contractData(r, 5) = sTVD(r): contractData(r, 6) = sNorth(r): contractData(r, 7) = sEast(r): contractData(r, 8) = sVS(r): contractData(r, 9) = sCross(r)
        contractData(r, 10) = sDLS(r): contractData(r, 11) = wsSurvey.Cells(r + 6, 5).Value2: contractData(r, 12) = surveyVisible(r, 20): contractData(r, 13) = wsSurvey.Cells(r + 6, 26).Value2
    Next r
    wsSurvey.Range("F7:Y506").ClearContents: wsSurvey.Range("F7").Resize(sCount, 20).Value2 = surveyVisible
    wsCalc.Range("T7:BS506").ClearContents: wsCalc.Range("T7").Resize(sCount, 18).Value2 = calcSurvey: wsCalc.Range("AL7").Resize(sCount, 34).Value2 = compareData
    wsResults.Range("A26:M525").ClearContents: wsResults.Range("A26").Resize(sCount, 13).Value2 = contractData

    nextTarget = WF_CalculateTargets(lengthFactor, vsa)
    WF_CalculateSlides lengthFactor, gradientFactor, angleFactor
    WF_CalculateFormations lengthFactor
    WF_WriteDirectionalGraphData lengthFactor, gradientFactor
    overallState = WF_WriteDirectionalChecks(maxDls, nextTarget)

    wsResults.Range("B6").Value2 = sMD(sCount) * lengthFactor
    wsResults.Range("B7").Value2 = latestCross * lengthFactor
    wsResults.Range("B8").Value2 = latest3d * lengthFactor
    wsResults.Range("B9").Value2 = maxDls * gradientFactor
    wsResults.Range("B10").Value2 = coverage
    wsResults.Range("B11").Value2 = nextTarget
    wsResults.Range("B12").Value2 = IIf(WF_CountText("Slide Performance", "S7:S206", "OK", False) > 0, "REVIEW", "OK")
    wsResults.Range("B13").Value2 = IIf(WF_CountNonBlank("Formation Tops", "D7:D106") = 0, "NO ACTUAL PICKS", "OK")
    wsResults.Range("B14").Value2 = "Latest valid survey": wsResults.Range("B15").Value2 = "DETERMINISTIC - NO UNCERTAINTY MODEL"
    wsResults.Range("B19").Value2 = latestCross * lengthFactor: wsResults.Range("B20").Value2 = Sqr((sNorth(sCount) - planN) ^ 2 + (sEast(sCount) - planE) ^ 2) * lengthFactor
    wsResults.Range("B21").Value2 = latest3d * lengthFactor

    WF_StatusCell wsSummary.Range("B5"), overallState
    wsSummary.Range("B6").Value2 = sMD(sCount) * lengthFactor: wsSummary.Range("B7").Value2 = latestCross * lengthFactor: wsSummary.Range("B8").Value2 = latest3d * lengthFactor
    wsSummary.Range("B9").Value2 = Format$(maxDls * gradientFactor, "0.00") & " / " & Format$(WF_DlsLimitSI() * gradientFactor, "0.00") & " " & WF_UnitLabel("Angular gradient")
    wsSummary.Range("B10").Value2 = nextTarget & " — " & IIf(overallState = "STOP", "Resolve STOP checks", IIf(overallState = "CAUTION", "Review cautions before proceeding", "Proceed under approved workflow"))
    wsSummary.Range("C6:C8").Value2 = WF_DDColumn3(WF_UnitLabel("Length"), WF_UnitLabel("Length"), WF_UnitLabel("Length"))
    WF_UpdateDirectionalHeaders
    WF_ConfigureDirectionalCharts
End Sub

Private Sub WF_CalculatePath(ByVal ws As Worksheet, ByVal lengthUnit As String, ByVal angleUnit As String, ByVal vsa As Double, ByRef countRows As Long, ByRef md() As Double, ByRef inc() As Double, ByRef azi() As Double, ByRef tvd() As Double, ByRef north() As Double, ByRef east() As Double, ByRef verticalSection() As Double, ByRef crossline() As Double, ByRef dls() As Double)
    Dim r As Long, dmd As Double, dogleg As Double, rf As Double, cosineDogleg As Double
    Dim dTvd As Double, dNorth As Double, dEast As Double, surfaceN As Double, surfaceE As Double
    countRows = 0
    For r = 7 To 506
        If Len(Trim$(CStr(ws.Cells(r, 2).Value2))) = 0 Then Exit For
        countRows = countRows + 1
    Next r
    ReDim md(1 To countRows): ReDim inc(1 To countRows): ReDim azi(1 To countRows): ReDim tvd(1 To countRows)
    ReDim north(1 To countRows): ReDim east(1 To countRows): ReDim verticalSection(1 To countRows): ReDim crossline(1 To countRows): ReDim dls(1 To countRows)
    surfaceN = WF_ToSI(WF_Num("Inputs", "B13"), lengthUnit): surfaceE = WF_ToSI(WF_Num("Inputs", "B14"), lengthUnit)
    For r = 1 To countRows
        md(r) = WF_ToSI(CDbl(ws.Cells(r + 6, 2).Value2), lengthUnit)
        inc(r) = WF_ToSI(CDbl(ws.Cells(r + 6, 3).Value2), angleUnit)
        azi(r) = WF_Mod2Pi(WF_ToSI(CDbl(ws.Cells(r + 6, 4).Value2), angleUnit))
        If inc(r) < 0# Or inc(r) > DD_PI Then Err.Raise vbObjectError + 8701, "WF_CalculatePath", ws.Name & " inclination outside 0..pi at row " & CStr(r + 6)
        If r = 1 Then
            tvd(r) = 0#: north(r) = surfaceN: east(r) = surfaceE: dls(r) = 0#
        Else
            dmd = md(r) - md(r - 1)
            If dmd <= 0# Then Err.Raise vbObjectError + 8702, "WF_CalculatePath", ws.Name & " MD must increase at row " & CStr(r + 6)
            cosineDogleg = Cos(inc(r - 1)) * Cos(inc(r)) + Sin(inc(r - 1)) * Sin(inc(r)) * Cos(azi(r) - azi(r - 1))
            dogleg = WorksheetFunction.Acos(WF_Clamp(cosineDogleg, -1#, 1#))
            rf = WF_RatioFactor(dogleg)
            dTvd = dmd / 2# * (Cos(inc(r - 1)) + Cos(inc(r))) * rf
            dNorth = dmd / 2# * (Sin(inc(r - 1)) * Cos(azi(r - 1)) + Sin(inc(r)) * Cos(azi(r))) * rf
            dEast = dmd / 2# * (Sin(inc(r - 1)) * Sin(azi(r - 1)) + Sin(inc(r)) * Sin(azi(r))) * rf
            tvd(r) = tvd(r - 1) + dTvd: north(r) = north(r - 1) + dNorth: east(r) = east(r - 1) + dEast: dls(r) = dogleg / dmd
        End If
        verticalSection(r) = north(r) * Cos(vsa) + east(r) * Sin(vsa)
        crossline(r) = -north(r) * Sin(vsa) + east(r) * Cos(vsa)
    Next r
End Sub

Private Function WF_RatioFactor(ByVal dogleg As Double) As Double
    If Abs(dogleg) < 0.000000001 Then WF_RatioFactor = 1# + dogleg ^ 2 / 12# + dogleg ^ 4 / 120# Else WF_RatioFactor = 2# * Tan(dogleg / 2#) / dogleg
End Function

Private Function WF_InterpolatePath(ByVal queryMd As Double, ByVal countRows As Long, ByRef md() As Double, ByRef inc() As Double, ByRef azi() As Double, ByRef tvd() As Double, ByRef north() As Double, ByRef east() As Double, ByRef outTvd As Double, ByRef outNorth As Double, ByRef outEast As Double, ByRef coverage As String) As Boolean
    Dim lower As Long, fraction As Double, dogleg As Double, partialDogleg As Double, partialMd As Double, rf As Double
    Dim n1 As Double, e1 As Double, v1 As Double, n2 As Double, e2 As Double, v2 As Double, ni As Double, ei As Double, vi As Double, norm As Double
    Dim interpInc As Double, interpAzi As Double, cosineDogleg As Double
    If queryMd < md(1) Then coverage = "BEFORE START": Exit Function
    If queryMd > md(countRows) Then coverage = "BEYOND TD": Exit Function
    If queryMd = md(countRows) Then outTvd = tvd(countRows): outNorth = north(countRows): outEast = east(countRows): coverage = "OK": WF_InterpolatePath = True: Exit Function
    For lower = 1 To countRows - 1
        If queryMd >= md(lower) And queryMd <= md(lower + 1) Then Exit For
    Next lower
    fraction = (queryMd - md(lower)) / (md(lower + 1) - md(lower))
    n1 = Sin(inc(lower)) * Cos(azi(lower)): e1 = Sin(inc(lower)) * Sin(azi(lower)): v1 = Cos(inc(lower))
    n2 = Sin(inc(lower + 1)) * Cos(azi(lower + 1)): e2 = Sin(inc(lower + 1)) * Sin(azi(lower + 1)): v2 = Cos(inc(lower + 1))
    cosineDogleg = WF_Clamp(n1 * n2 + e1 * e2 + v1 * v2, -1#, 1#): dogleg = WorksheetFunction.Acos(cosineDogleg)
    If Abs(dogleg) < 0.000000001 Then
        ni = (1# - fraction) * n1 + fraction * n2: ei = (1# - fraction) * e1 + fraction * e2: vi = (1# - fraction) * v1 + fraction * v2
        norm = Sqr(ni ^ 2 + ei ^ 2 + vi ^ 2): ni = ni / norm: ei = ei / norm: vi = vi / norm
    Else
        ni = Sin((1# - fraction) * dogleg) / Sin(dogleg) * n1 + Sin(fraction * dogleg) / Sin(dogleg) * n2
        ei = Sin((1# - fraction) * dogleg) / Sin(dogleg) * e1 + Sin(fraction * dogleg) / Sin(dogleg) * e2
        vi = Sin((1# - fraction) * dogleg) / Sin(dogleg) * v1 + Sin(fraction * dogleg) / Sin(dogleg) * v2
    End If
    interpInc = WorksheetFunction.Acos(WF_Clamp(vi, -1#, 1#))
    If Sqr(ni ^ 2 + ei ^ 2) < 0.000000001 Then interpAzi = azi(lower) Else interpAzi = WF_Mod2Pi(WorksheetFunction.Atan2(ni, ei))
    partialMd = queryMd - md(lower): partialDogleg = fraction * dogleg: rf = WF_RatioFactor(partialDogleg)
    outTvd = tvd(lower) + partialMd / 2# * (Cos(inc(lower)) + Cos(interpInc)) * rf
    outNorth = north(lower) + partialMd / 2# * (Sin(inc(lower)) * Cos(azi(lower)) + Sin(interpInc) * Cos(interpAzi)) * rf
    outEast = east(lower) + partialMd / 2# * (Sin(inc(lower)) * Sin(azi(lower)) + Sin(interpInc) * Sin(interpAzi)) * rf
    coverage = "OK": WF_InterpolatePath = True
End Function

Private Sub WF_FillCanonicalPathRow(ByRef output() As Variant, ByVal r As Long, ByVal stationId As Variant, ByRef md() As Double, ByRef inc() As Double, ByRef azi() As Double, ByRef tvd() As Double, ByRef north() As Double, ByRef east() As Double, ByRef verticalSection() As Double, ByRef crossline() As Double, ByRef dls() As Double)
    Dim dmd As Double, dogleg As Double, rf As Double
    If r > 1 Then dmd = md(r) - md(r - 1): dogleg = dls(r) * dmd: rf = WF_RatioFactor(dogleg) Else rf = 1#
    output(r, 1) = True: output(r, 2) = stationId: output(r, 3) = md(r): output(r, 4) = inc(r): output(r, 5) = azi(r)
    output(r, 6) = dmd: output(r, 7) = dogleg: output(r, 8) = rf
    If r > 1 Then output(r, 9) = tvd(r) - tvd(r - 1): output(r, 10) = north(r) - north(r - 1): output(r, 11) = east(r) - east(r - 1)
    output(r, 12) = tvd(r): output(r, 13) = north(r): output(r, 14) = east(r): output(r, 15) = verticalSection(r): output(r, 16) = crossline(r): output(r, 17) = dls(r): output(r, 18) = "OK"
End Sub

Private Function WF_CalculateTargets(ByVal lengthFactor As Double, ByVal vsa As Double) As String
    Dim ws As Worksheet, r As Long, md As Double, actualTvd As Double, actualN As Double, actualE As Double, coverage As String
    Dim centerN As Double, centerE As Double, centerTvd As Double, major As Double, minor As Double, verticalTolerance As Double, theta As Double
    Dim dn As Double, de As Double, dt As Double, localMajor As Double, localMinor As Double, envelope As Double, verticalUtil As Double, targetType As String, status As String
    Set ws = ThisWorkbook.Worksheets("Targets")
    ws.Range("L7:Q106").ClearContents
    WF_CalculateTargets = "NO TARGET"
    For r = 7 To 106
        If Len(Trim$(CStr(ws.Cells(r, 1).Value2))) = 0 Then Exit For
        md = WF_ToSI(CDbl(ws.Cells(r, 2).Value2), WF_Str("Inputs", "E9", "m"))
        centerN = WF_ToSI(CDbl(ws.Cells(r, 3).Value2), WF_Str("Inputs", "E9", "m")): centerE = WF_ToSI(CDbl(ws.Cells(r, 4).Value2), WF_Str("Inputs", "E9", "m"))
        centerTvd = WF_ToSI(CDbl(ws.Cells(r, 5).Value2), WF_Str("Inputs", "E9", "m")): targetType = CStr(ws.Cells(r, 6).Value2)
        major = WF_ToSI(CDbl(ws.Cells(r, 7).Value2), WF_Str("Inputs", "E9", "m")): minor = WF_ToSI(CDbl(ws.Cells(r, 8).Value2), WF_Str("Inputs", "E9", "m"))
        theta = WF_DegToRad(CDbl(ws.Cells(r, 9).Value2)): verticalTolerance = WF_ToSI(CDbl(ws.Cells(r, 10).Value2), WF_Str("Inputs", "E9", "m"))
        If major <= 0# Or (targetType = "Ellipse" And minor <= 0#) Or verticalTolerance < 0# Then
            status = "INVALID GEOMETRY": ws.Cells(r, 17).Value2 = status
        ElseIf WF_InterpolatePath(md, sCount, sMD, sInc, sAzi, sTVD, sNorth, sEast, actualTvd, actualN, actualE, coverage) Then
            dn = actualN - centerN: de = actualE - centerE: dt = actualTvd - centerTvd
            localMajor = dn * Cos(theta) + de * Sin(theta): localMinor = -dn * Sin(theta) + de * Cos(theta)
            If targetType = "Ellipse" Then envelope = Sqr((localMajor / major) ^ 2 + (localMinor / minor) ^ 2) Else envelope = Sqr(dn ^ 2 + de ^ 2) / major
            If verticalTolerance = 0# Then verticalUtil = IIf(Abs(dt) < 0.000000001, 0#, 1E+99) Else verticalUtil = Abs(dt) / verticalTolerance
            status = IIf(envelope <= 1# And verticalUtil <= 1#, "ACTUAL HIT", "ACTUAL MISS")
            ws.Cells(r, 12).Value2 = "ACTUAL": ws.Cells(r, 13).Value2 = localMajor * lengthFactor: ws.Cells(r, 14).Value2 = localMinor * lengthFactor
            ws.Cells(r, 15).Value2 = envelope: ws.Cells(r, 16).Value2 = verticalUtil: ws.Cells(r, 17).Value2 = status
        Else
            status = "NOT REACHED": ws.Cells(r, 12).Value2 = coverage: ws.Cells(r, 17).Value2 = status
        End If
        If WF_CalculateTargets = "NO TARGET" Then WF_CalculateTargets = CStr(ws.Cells(r, 1).Value2) & ": " & status
    Next r
End Function

Private Sub WF_CalculateSlides(ByVal lengthFactor As Double, ByVal gradientFactor As Double, ByVal angleFactor As Double)
    Dim ws As Worksheet, r As Long, mdIn As Double, mdOut As Double, inTvd As Double, inN As Double, inE As Double, outTvd As Double, outN As Double, outE As Double
    Dim coverageIn As String, coverageOut As String, lower As Long, incIn As Double, aziIn As Double, incOut As Double, aziOut As Double, course As Double
    Dim build As Double, turn As Double, residualBuild As Double, residualTurn As Double, slideLength As Double, yieldValue As Double, toolface As Double, commanded As Double
    Set ws = ThisWorkbook.Worksheets("Slide Performance"): ws.Range("K7:S206").ClearContents
    For r = 7 To 206
        If Len(Trim$(CStr(ws.Cells(r, 1).Value2))) = 0 Then Exit For
        mdIn = WF_ToSI(CDbl(ws.Cells(r, 3).Value2), WF_Str("Inputs", "E10", "m")): mdOut = WF_ToSI(CDbl(ws.Cells(r, 4).Value2), WF_Str("Inputs", "E10", "m"))
        course = mdOut - mdIn: slideLength = WF_ToSI(CDbl(ws.Cells(r, 5).Value2), WF_Str("Inputs", "E10", "m"))
        If course <= 0# Or slideLength <= 0# Then ws.Cells(r, 19).Value2 = "INVALID INTERVAL": GoTo NextSlide
        If Not WF_DirectionAtMd(mdIn, incIn, aziIn, coverageIn) Or Not WF_DirectionAtMd(mdOut, incOut, aziOut, coverageOut) Then ws.Cells(r, 19).Value2 = "OUTSIDE SURVEY": GoTo NextSlide
        build = (incOut - incIn) / course
        turn = WF_WrapPi(aziOut - aziIn) * Sin((incIn + incOut) / 2#) / course
        residualBuild = (build - WF_OptionalGradient(ws.Cells(r, 8).Value2)) * course / slideLength
        residualTurn = (turn - WF_OptionalGradient(ws.Cells(r, 9).Value2)) * course / slideLength
        yieldValue = Sqr(residualBuild ^ 2 + residualTurn ^ 2)
        If yieldValue > 0.000000001 Then toolface = WF_Mod2Pi(WorksheetFunction.Atan2(residualBuild, residualTurn))
        commanded = WF_ToSI(CDbl(ws.Cells(r, 7).Value2), WF_Str("Inputs", "E8", "rad"))
        ws.Cells(r, 11).Value2 = build * gradientFactor: ws.Cells(r, 12).Value2 = turn * gradientFactor
        ws.Cells(r, 13).Value2 = residualBuild * gradientFactor: ws.Cells(r, 14).Value2 = residualTurn * gradientFactor
        ws.Cells(r, 15).Value2 = yieldValue * gradientFactor: ws.Cells(r, 16).Value2 = toolface * angleFactor: ws.Cells(r, 17).Value2 = WF_WrapPi(toolface - commanded) * angleFactor
        ws.Cells(r, 18).Value2 = WF_RollingSlideYield(ws, r): ws.Cells(r, 19).Value2 = IIf((incIn + incOut) / 2# < WF_DegToRad(WF_Num("Inputs", "N5")), "LOW INCLINATION", IIf(slideLength < WF_ToSI(WF_Num("Inputs", "N6"), WF_Str("Inputs", "E10", "m")), "SHORT SLIDE", IIf(yieldValue > WF_ToSI(WF_Num("Inputs", "N7"), WF_Str("Inputs", "K9", "deg/100ft")), "OUTLIER", "OK")))
NextSlide:
    Next r
End Sub

Private Function WF_DirectionAtMd(ByVal queryMd As Double, ByRef outInc As Double, ByRef outAzi As Double, ByRef coverage As String) As Boolean
    Dim lower As Long, fraction As Double, n1 As Double, e1 As Double, v1 As Double, n2 As Double, e2 As Double, v2 As Double, ni As Double, ei As Double, vi As Double, dogleg As Double, norm As Double
    If queryMd < sMD(1) Then coverage = "BEFORE START": Exit Function
    If queryMd > sMD(sCount) Then coverage = "BEYOND TD": Exit Function
    If queryMd = sMD(sCount) Then outInc = sInc(sCount): outAzi = sAzi(sCount): coverage = "OK": WF_DirectionAtMd = True: Exit Function
    For lower = 1 To sCount - 1: If queryMd >= sMD(lower) And queryMd <= sMD(lower + 1) Then Exit For
    Next lower
    fraction = (queryMd - sMD(lower)) / (sMD(lower + 1) - sMD(lower))
    n1 = Sin(sInc(lower)) * Cos(sAzi(lower)): e1 = Sin(sInc(lower)) * Sin(sAzi(lower)): v1 = Cos(sInc(lower))
    n2 = Sin(sInc(lower + 1)) * Cos(sAzi(lower + 1)): e2 = Sin(sInc(lower + 1)) * Sin(sAzi(lower + 1)): v2 = Cos(sInc(lower + 1))
    dogleg = WorksheetFunction.Acos(WF_Clamp(n1 * n2 + e1 * e2 + v1 * v2, -1#, 1#))
    If Abs(dogleg) < 0.000000001 Then
        ni = (1# - fraction) * n1 + fraction * n2: ei = (1# - fraction) * e1 + fraction * e2: vi = (1# - fraction) * v1 + fraction * v2
        norm = Sqr(ni ^ 2 + ei ^ 2 + vi ^ 2): ni = ni / norm: ei = ei / norm: vi = vi / norm
    Else
        ni = Sin((1# - fraction) * dogleg) / Sin(dogleg) * n1 + Sin(fraction * dogleg) / Sin(dogleg) * n2
        ei = Sin((1# - fraction) * dogleg) / Sin(dogleg) * e1 + Sin(fraction * dogleg) / Sin(dogleg) * e2
        vi = Sin((1# - fraction) * dogleg) / Sin(dogleg) * v1 + Sin(fraction * dogleg) / Sin(dogleg) * v2
    End If
    outInc = WorksheetFunction.Acos(WF_Clamp(vi, -1#, 1#)): If Sqr(ni ^ 2 + ei ^ 2) < 0.000000001 Then outAzi = sAzi(lower) Else outAzi = WF_Mod2Pi(WorksheetFunction.Atan2(ni, ei))
    coverage = "OK": WF_DirectionAtMd = True
End Function

Private Function WF_OptionalGradient(ByVal value As Variant) As Double
    If IsNumeric(value) And Len(Trim$(CStr(value))) > 0 Then WF_OptionalGradient = WF_ToSI(CDbl(value), WF_Str("Inputs", "K9", "rad/m"))
End Function

Private Function WF_RollingSlideYield(ByVal ws As Worksheet, ByVal currentRow As Long) As Variant
    Dim windowSize As Long, firstRow As Long, r As Long, numerator As Double, denominator As Double
    windowSize = CLng(WF_Num("Inputs", "N8", 3)): firstRow = Application.Max(7, currentRow - windowSize + 1)
    For r = firstRow To currentRow
        If CStr(ws.Cells(r, 19).Value2) = "OK" Or r = currentRow Then
            If IsNumeric(ws.Cells(r, 5).Value2) And IsNumeric(ws.Cells(r, 15).Value2) Then numerator = numerator + CDbl(ws.Cells(r, 5).Value2) * CDbl(ws.Cells(r, 15).Value2): denominator = denominator + CDbl(ws.Cells(r, 5).Value2)
        End If
    Next r
    If denominator > 0# Then WF_RollingSlideYield = numerator / denominator Else WF_RollingSlideYield = Empty
End Function

Private Sub WF_CalculateFormations(ByVal lengthFactor As Double)
    Dim ws As Worksheet, r As Long, pickMd As Double, actualTvd As Double, actualN As Double, actualE As Double, coverage As String, prognosis As Double, tolerance As Double, highLow As Double
    Set ws = ThisWorkbook.Worksheets("Formation Tops"): ws.Range("G7:K106").ClearContents
    For r = 7 To 106
        If Len(Trim$(CStr(ws.Cells(r, 1).Value2))) = 0 Then Exit For
        If Len(Trim$(CStr(ws.Cells(r, 4).Value2))) > 0 And IsNumeric(ws.Cells(r, 4).Value2) Then
            pickMd = WF_ToSI(CDbl(ws.Cells(r, 4).Value2), WF_Str("Inputs", "E11", "m"))
            If WF_InterpolatePath(pickMd, sCount, sMD, sInc, sAzi, sTVD, sNorth, sEast, actualTvd, actualN, actualE, coverage) Then
                prognosis = WF_ToSI(CDbl(ws.Cells(r, 3).Value2), WF_Str("Inputs", "E11", "m")): highLow = prognosis - actualTvd
                ws.Cells(r, 7).Value2 = actualTvd * lengthFactor: ws.Cells(r, 8).Value2 = highLow * lengthFactor: ws.Cells(r, 9).Value2 = IIf(highLow > 0#, "HIGH", IIf(highLow < 0#, "LOW", "ON PROGNOSIS")): ws.Cells(r, 10).Value2 = "OK"
                If Len(Trim$(CStr(ws.Cells(r, 5).Value2))) > 0 Then tolerance = WF_ToSI(CDbl(ws.Cells(r, 5).Value2), WF_Str("Inputs", "E11", "m")) Else tolerance = 1E+99
                ws.Cells(r, 11).Value2 = IIf(Abs(highLow) <= tolerance, "OK", "OUTSIDE TOLERANCE")
            Else
                ws.Cells(r, 10).Value2 = coverage: ws.Cells(r, 11).Value2 = coverage
            End If
        End If
    Next r
End Sub

Private Sub WF_WriteDirectionalGraphData(ByVal lengthFactor As Double, ByVal gradientFactor As Double)
    Dim ws As Worksheet, r As Long, maxRows As Long, planTvd As Double, planN As Double, planE As Double, coverage As String, slideRows As Long, targetRows As Long
    Set ws = ThisWorkbook.Worksheets("Calc")
    ws.Range("DA7:EG506").ClearContents: maxRows = Application.Max(pCount, sCount)
    For r = 1 To maxRows
        If r <= pCount Then
            ws.Cells(r + 6, 105).Value2 = pEast(r) * lengthFactor: ws.Cells(r + 6, 106).Value2 = pNorth(r) * lengthFactor
            ws.Cells(r + 6, 109).Value2 = pVS(r) * lengthFactor: ws.Cells(r + 6, 110).Value2 = pTVD(r) * lengthFactor
            ws.Cells(r + 6, 117).Value2 = pDLS(r) * gradientFactor
        End If
        If r <= sCount Then
            ws.Cells(r + 6, 107).Value2 = sEast(r) * lengthFactor: ws.Cells(r + 6, 108).Value2 = sNorth(r) * lengthFactor
            ws.Cells(r + 6, 111).Value2 = sVS(r) * lengthFactor: ws.Cells(r + 6, 112).Value2 = sTVD(r) * lengthFactor
            ws.Cells(r + 6, 113).Value2 = sMD(r) * lengthFactor: ws.Cells(r + 6, 114).Value2 = sInc(r) * WF_UnitFactor("Angle"): ws.Cells(r + 6, 115).Value2 = sAzi(r) * WF_UnitFactor("Angle")
            ws.Cells(r + 6, 116).Value2 = sMD(r) * lengthFactor: ws.Cells(r + 6, 118).Value2 = sDLS(r) * gradientFactor
            ws.Cells(r + 6, 121).Value2 = sMD(r) * lengthFactor
            If WF_InterpolatePath(sMD(r), pCount, pMD, pInc, pAzi, pTVD, pNorth, pEast, planTvd, planN, planE, coverage) Then
                ws.Cells(r + 6, 122).Value2 = (sTVD(r) - planTvd) * lengthFactor: ws.Cells(r + 6, 123).Value2 = (sVS(r) - (planN * Cos(WF_DegToRad(WF_Num("Inputs", "B16"))) + planE * Sin(WF_DegToRad(WF_Num("Inputs", "B16"))))) * lengthFactor
                ws.Cells(r + 6, 124).Value2 = ((sNorth(r) - planN) * Cos(WF_DegToRad(WF_Num("Inputs", "B16"))) + (sEast(r) - planE) * Sin(WF_DegToRad(WF_Num("Inputs", "B16")))) * lengthFactor
                ws.Cells(r + 6, 125).Value2 = (-(sNorth(r) - planN) * Sin(WF_DegToRad(WF_Num("Inputs", "B16"))) + (sEast(r) - planE) * Cos(WF_DegToRad(WF_Num("Inputs", "B16")))) * lengthFactor
                ws.Cells(r + 6, 127).Value2 = sMD(r) * lengthFactor: ws.Cells(r + 6, 128).Value2 = Sqr((sNorth(r) - planN) ^ 2 + (sEast(r) - planE) ^ 2) * lengthFactor
                ws.Cells(r + 6, 129).Value2 = Sqr((sNorth(r) - planN) ^ 2 + (sEast(r) - planE) ^ 2 + (sTVD(r) - planTvd) ^ 2) * lengthFactor
            End If
        End If
    Next r
    For r = 7 To 206
        If Len(Trim$(CStr(ThisWorkbook.Worksheets("Slide Performance").Cells(r, 1).Value2))) = 0 Then Exit For
        ws.Cells(r, 131).Value2 = ThisWorkbook.Worksheets("Slide Performance").Cells(r, 1).Value2: ws.Cells(r, 132).Value2 = ThisWorkbook.Worksheets("Slide Performance").Cells(r, 15).Value2: ws.Cells(r, 133).Value2 = WF_Num("Inputs", "N7")
    Next r
    For r = 7 To 106
        If Len(Trim$(CStr(ThisWorkbook.Worksheets("Targets").Cells(r, 1).Value2))) = 0 Then Exit For
        ws.Cells(r, 135).Value2 = ThisWorkbook.Worksheets("Targets").Cells(r, 1).Value2: ws.Cells(r, 136).Value2 = ThisWorkbook.Worksheets("Targets").Cells(r, 15).Value2: ws.Cells(r, 137).Value2 = ThisWorkbook.Worksheets("Targets").Cells(r, 16).Value2
    Next r
End Sub

Private Function WF_WriteDirectionalChecks(ByVal maxDls As Double, ByVal nextTarget As String) As String
    Dim ws As Worksheet, limit As Double, invalidTargets As Long, slideIssues As Long, formationBeyond As Long, r As Long, severity As String, cautionCount As Long, stopCount As Long
    Set ws = ThisWorkbook.Worksheets("Checks"): limit = WF_DlsLimitSI()
    ws.Range("B6:D25").ClearContents
    WF_SetCheck ws, 6, "Defined", "PASS", "INFO": WF_SetCheck ws, 7, "Defined", "PASS", "INFO"
    WF_SetCheck ws, 8, CStr(pCount) & " / 500", "PASS", "INFO": WF_SetCheck ws, 9, CStr(sCount) & " / 500", "PASS", "INFO"
    WF_SetCheck ws, 10, 0, "PASS", "INFO": WF_SetCheck ws, 11, 0, "PASS", "INFO": WF_SetCheck ws, 12, 0, "PASS", "INFO"
    WF_SetCheck ws, 13, WF_MaxGap(sMD, sCount) * WF_UnitFactor("Length"), IIf(WF_MaxGap(sMD, sCount) > WF_ToSI(WF_Num("Inputs", "N9"), WF_Str("Inputs", "E7", "m")), "CAUTION", "PASS"), IIf(WF_MaxGap(sMD, sCount) > WF_ToSI(WF_Num("Inputs", "N9"), WF_Str("Inputs", "E7", "m")), "CAUTION", "INFO")
    WF_SetCheck ws, 14, IIf(sMD(sCount) <= pMD(pCount), "OK", "BEYOND TD"), IIf(sMD(sCount) <= pMD(pCount), "PASS", "CAUTION"), IIf(sMD(sCount) <= pMD(pCount), "INFO", "CAUTION")
    WF_SetCheck ws, 15, maxDls * WF_UnitFactor("Angular gradient"), IIf(maxDls <= limit, "PASS", "CAUTION"), IIf(maxDls <= limit, "INFO", "CAUTION")
    invalidTargets = WF_CountContains("Targets", "Q7:Q106", "INVALID"): slideIssues = WF_CountText("Slide Performance", "S7:S206", "OK", False): formationBeyond = WF_CountContains("Formation Tops", "J7:J106", "BEYOND")
    WF_SetCheck ws, 16, invalidTargets, IIf(invalidTargets = 0, "PASS", "STOP"), IIf(invalidTargets = 0, "INFO", "STOP")
    WF_SetCheck ws, 17, slideIssues, IIf(slideIssues = 0, "PASS", "CAUTION"), IIf(slideIssues = 0, "INFO", "CAUTION")
    WF_SetCheck ws, 18, formationBeyond, IIf(formationBeyond = 0, "PASS", "CAUTION"), IIf(formationBeyond = 0, "INFO", "CAUTION")
    WF_SetCheck ws, 19, 0, "PASS", "INFO": WF_SetCheck ws, 20, "VBA engine", "PASS", "INFO"
    For r = 21 To 25: WF_SetCheck ws, r, IIf(r = 25, invalidTargets, "Not calculated"), "INFO", "INFO": Next r
    For r = 6 To 25
        severity = CStr(ws.Cells(r, 4).Value2)
        If severity = "STOP" Then
            stopCount = stopCount + 1
        ElseIf severity = "CAUTION" Then
            cautionCount = cautionCount + 1
        End If
    Next r
    If stopCount > 0 Then
        WF_WriteDirectionalChecks = "STOP"
    ElseIf cautionCount > 0 Then
        WF_WriteDirectionalChecks = "CAUTION"
    Else
        WF_WriteDirectionalChecks = "READY"
    End If
End Function

Private Sub WF_SetCheck(ByVal ws As Worksheet, ByVal rowIndex As Long, ByVal measured As Variant, ByVal status As String, ByVal severity As String)
    ws.Cells(rowIndex, 2).Value2 = measured: ws.Cells(rowIndex, 3).Value2 = status: ws.Cells(rowIndex, 4).Value2 = severity
End Sub

Private Function WF_DlsLimitSI() As Double
    WF_DlsLimitSI = WF_ToSI(WF_Num("Inputs", "H5"), WF_Str("Inputs", "H6", "rad/m"))
End Function

Private Function WF_WrapPi(ByVal angle As Double) As Double
    WF_WrapPi = WF_Mod2Pi(angle + DD_PI) - DD_PI
End Function

Private Function WF_MaxGap(ByRef md() As Double, ByVal countRows As Long) As Double
    Dim r As Long, gap As Double: For r = 2 To countRows: gap = md(r) - md(r - 1): If gap > WF_MaxGap Then WF_MaxGap = gap
    Next r
End Function

Private Function WF_CountNonBlank(ByVal sheetName As String, ByVal address As String) As Long
    WF_CountNonBlank = Application.WorksheetFunction.CountA(ThisWorkbook.Worksheets(sheetName).Range(address))
End Function

Private Function WF_CountContains(ByVal sheetName As String, ByVal address As String, ByVal textValue As String) As Long
    Dim cell As Range: For Each cell In ThisWorkbook.Worksheets(sheetName).Range(address).Cells: If InStr(1, CStr(cell.Value2), textValue, vbTextCompare) > 0 Then WF_CountContains = WF_CountContains + 1
    Next cell
End Function

Private Function WF_CountText(ByVal sheetName As String, ByVal address As String, ByVal expected As String, ByVal countMatches As Boolean) As Long
    Dim cell As Range, value As String
    For Each cell In ThisWorkbook.Worksheets(sheetName).Range(address).Cells
        value = Trim$(CStr(cell.Value2))
        If Len(value) > 0 Then
            If countMatches And StrComp(value, expected, vbTextCompare) = 0 Then WF_CountText = WF_CountText + 1
            If Not countMatches And StrComp(value, expected, vbTextCompare) <> 0 Then WF_CountText = WF_CountText + 1
        End If
    Next cell
End Function

Private Sub WF_UpdateDirectionalHeaders()
    Dim lengthUnit As String, angleUnit As String, gradientUnit As String
    lengthUnit = WF_UnitLabel("Length"): angleUnit = WF_UnitLabel("Angle"): gradientUnit = WF_UnitLabel("Angular gradient")
    ThisWorkbook.Worksheets("Plan").Range("G5:K5").Value2 = WF_DDRow5(lengthUnit, lengthUnit, lengthUnit, lengthUnit, lengthUnit)
    ThisWorkbook.Worksheets("Plan").Range("L5").Value2 = gradientUnit
    ThisWorkbook.Worksheets("Survey").Range("G5:V5").Value2 = WF_DDRepeatedRow(16, lengthUnit): ThisWorkbook.Worksheets("Survey").Range("X5").Value2 = gradientUnit
    ThisWorkbook.Worksheets("Targets").Range("M5:N5").Value2 = WF_DDRow2(lengthUnit, lengthUnit)
    ThisWorkbook.Worksheets("Formation Tops").Range("G5:H5").Value2 = WF_DDRow2(lengthUnit, lengthUnit)
    ThisWorkbook.Worksheets("Slide Performance").Range("K5:O5").Value2 = WF_DDRepeatedRow(5, gradientUnit)
    ThisWorkbook.Worksheets("Slide Performance").Range("P5:Q5").Value2 = WF_DDRow2(angleUnit, angleUnit): ThisWorkbook.Worksheets("Slide Performance").Range("R5").Value2 = gradientUnit
    With ThisWorkbook.Worksheets("Calc")
        .Range("DA6").Value2 = "Plan East " & lengthUnit: .Range("DB6").Value2 = "Plan North " & lengthUnit
        .Range("DC6").Value2 = "Survey East " & lengthUnit: .Range("DD6").Value2 = "Survey North " & lengthUnit
        .Range("DE6").Value2 = "Plan VS " & lengthUnit: .Range("DF6").Value2 = "Plan TVD " & lengthUnit
        .Range("DG6").Value2 = "Survey VS " & lengthUnit: .Range("DH6").Value2 = "Survey TVD " & lengthUnit
        .Range("DI6").Value2 = "MD " & lengthUnit: .Range("DJ6").Value2 = "Inclination " & angleUnit: .Range("DK6").Value2 = "Azimuth " & angleUnit
        .Range("DM6").Value2 = "MD " & lengthUnit: .Range("DN6").Value2 = "Plan DLS " & gradientUnit: .Range("DO6").Value2 = "Survey DLS " & gradientUnit
        .Range("DQ6").Value2 = "MD " & lengthUnit: .Range("DR6").Value2 = "dTVD " & lengthUnit: .Range("DS6").Value2 = "dVS " & lengthUnit
        .Range("DT6").Value2 = "Along " & lengthUnit: .Range("DU6").Value2 = "Crossline " & lengthUnit
        .Range("DW6").Value2 = "MD " & lengthUnit: .Range("DX6").Value2 = "Horizontal Error " & lengthUnit: .Range("DY6").Value2 = "3D Error " & lengthUnit
    End With
End Sub

Private Sub WF_ConfigureDirectionalCharts()
    Dim wsGraphs As Worksheet
    Dim lengthUnit As String, angleUnit As String, gradientUnit As String
    Set wsGraphs = ThisWorkbook.Worksheets("Graphs")
    lengthUnit = WF_UnitLabel("Length"): angleUnit = WF_UnitLabel("Angle"): gradientUnit = WF_UnitLabel("Angular gradient")
    WF_ConfigureChartAxes wsGraphs.Name, 1, "East (" & lengthUnit & ")", "North (" & lengthUnit & ")"
    WF_ConfigureDepthChart wsGraphs.Name, 2, "Vertical section (" & lengthUnit & ")", "TVD (" & lengthUnit & ")"
    WF_ConfigureDepthChart wsGraphs.Name, 3, "Angle (" & angleUnit & ")", "MD (" & lengthUnit & ")"
    WF_ConfigureDepthChart wsGraphs.Name, 4, "DLS (" & gradientUnit & ")", "MD (" & lengthUnit & ")"
    WF_ConfigureDepthChart wsGraphs.Name, 5, "Position error (" & lengthUnit & ")", "MD (" & lengthUnit & ")"
    WF_ConfigureDepthChart wsGraphs.Name, 6, "Position error (" & lengthUnit & ")", "MD (" & lengthUnit & ")"
    WF_ConfigureChartAxes wsGraphs.Name, 7, "Stand", "Slide yield (" & gradientUnit & ")"
    WF_ConfigureChartAxes wsGraphs.Name, 8, "Target", "Utilization"
    wsGraphs.ChartObjects(1).Chart.ChartTitle.Text = "Plan View — East vs North (" & lengthUnit & ")"
    wsGraphs.ChartObjects(2).Chart.ChartTitle.Text = "Vertical Section — VS vs TVD (" & lengthUnit & ")"
End Sub

Private Function WF_DDRepeatedRow(ByVal countValues As Long, ByVal value As Variant) As Variant
    Dim output() As Variant, c As Long: ReDim output(1 To 1, 1 To countValues): For c = 1 To countValues: output(1, c) = value: Next c: WF_DDRepeatedRow = output
End Function

Private Function WF_DDRow2(ByVal a As Variant, ByVal b As Variant) As Variant
    Dim output(1 To 1, 1 To 2) As Variant: output(1, 1) = a: output(1, 2) = b: WF_DDRow2 = output
End Function

Private Function WF_DDColumn3(ByVal a As Variant, ByVal b As Variant, ByVal c As Variant) As Variant
    Dim output(1 To 3, 1 To 1) As Variant: output(1, 1) = a: output(2, 1) = b: output(3, 1) = c: WF_DDColumn3 = output
End Function

Private Function WF_DDRow5(ByVal a As Variant, ByVal b As Variant, ByVal c As Variant, ByVal d As Variant, ByVal e As Variant) As Variant
    Dim output(1 To 1, 1 To 5) As Variant: output(1, 1) = a: output(1, 2) = b: output(1, 3) = c: output(1, 4) = d: output(1, 5) = e: WF_DDRow5 = output
End Function
