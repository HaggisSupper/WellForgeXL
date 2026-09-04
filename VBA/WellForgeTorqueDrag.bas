Attribute VB_Name = "WellForgeTorqueDrag"
Option Explicit

Private Const TD_PI As Double = 3.14159265358979
Private Const TD_G As Double = 9.80665

Public Sub WF_CalcTorqueDrag()
    WF_RunTorqueDragRustEngine
End Sub

Private Sub WF_CalcTorqueDragLegacy()
    Dim wsIn As Worksheet, wsSurvey As Worksheet, wsCalc As Worksheet, wsResults As Worksheet
    Dim wsSummary As Worksheet, wsGraphs As Worksheet, wsAll As Worksheet, wsOps As Worksheet
    Dim countRows As Long, r As Long, opIndex As Long, colIndex As Long
    Dim calcData() As Variant, resultData() As Variant, allData() As Variant
    Dim md As Double, prevMd As Double, dmd As Double, inc As Double, prevInc As Double, azi As Double, prevAzi As Double
    Dim dogleg As Double, cosineDogleg As Double, buoyedWeight As Double, normalForce As Double, drag As Double
    Dim pooh As Double, rih As Double, slideTorque As Double, rotateTorque As Double, backreamTorque As Double
    Dim sinusoidal As Double, helical As Double, youngPa As Double, inertia As Double
    Dim density As Double, frictionFactor As Double, wob As Double, surfaceTorque As Double, od As Double, insideDiameter As Double, steelDensity As Double
    Dim lengthFactor As Double, forceFactor As Double, torqueFactor As Double, angleFactor As Double
    Dim minRih As Double, peakPooh As Double, governingDepth As Double
    Dim operationNames As Variant, axialColumns As Variant, torqueColumns As Variant
    Dim opData() As Variant, helperRow As Long, lastOutputRow As Long

    Set wsIn = ThisWorkbook.Worksheets("Inputs")
    Set wsSurvey = ThisWorkbook.Worksheets("Survey")
    Set wsCalc = ThisWorkbook.Worksheets("Calc")
    Set wsResults = ThisWorkbook.Worksheets("Results")
    Set wsSummary = ThisWorkbook.Worksheets("Summary")
    Set wsGraphs = ThisWorkbook.Worksheets("Graphs")
    Set wsAll = ThisWorkbook.Worksheets("ALL")
    Set wsOps = ThisWorkbook.Worksheets("Operation Charts")

    countRows = WF_ContiguousRows(wsSurvey, 6, 1, 500)
    If countRows < 2 Then Err.Raise vbObjectError + 8500, "WF_CalcTorqueDrag", "At least two survey stations are required"
    lastOutputRow = 5 + countRows

    density = WF_Num("Inputs", "B5")
    frictionFactor = WF_Num("Inputs", "B6")
    wob = WF_Num("Inputs", "B7")
    surfaceTorque = WF_Num("Inputs", "B8")
    od = WF_Num("Inputs", "B9")
    insideDiameter = WF_Num("Inputs", "B10")
    steelDensity = WF_Num("Inputs", "B11", 7850#)
    youngPa = WF_ToSI(WF_Num("Inputs", "B12"), WF_Str("Inputs", "C12", "GPa"))
    If od <= insideDiameter Or insideDiameter < 0# Or youngPa <= 0# Or steelDensity <= 0# Then Err.Raise vbObjectError + 8501, "WF_CalcTorqueDrag", "Invalid tubular or material properties"

    lengthFactor = WF_UnitFactor("Length")
    forceFactor = WF_UnitFactor("Force")
    torqueFactor = WF_UnitFactor("Torque")
    angleFactor = WF_UnitFactor("Angle")
    buoyedWeight = steelDensity * TD_G * TD_PI / 4# * (od ^ 2 - insideDiameter ^ 2) * (1# - density / steelDensity)
    inertia = TD_PI / 64# * (od ^ 4 - insideDiameter ^ 4)

    ReDim calcData(1 To countRows, 1 To 14)
    ReDim resultData(1 To countRows, 1 To 11)
    ReDim allData(1 To countRows, 1 To 14)
    minRih = 1E+99

    For r = 1 To countRows
        md = CDbl(wsSurvey.Cells(r + 5, 1).Value2)
        inc = CDbl(wsSurvey.Cells(r + 5, 2).Value2)
        azi = CDbl(wsSurvey.Cells(r + 5, 3).Value2)
        If r = 1 Then
            dmd = 0#: dogleg = 0#: pooh = wob: rih = wob
        Else
            dmd = md - prevMd
            If dmd <= 0# Then Err.Raise vbObjectError + 8502, "WF_CalcTorqueDrag", "Survey MD must increase at row " & CStr(r + 5)
            cosineDogleg = Cos(prevInc) * Cos(inc) + Sin(prevInc) * Sin(inc) * Cos(azi - prevAzi)
            dogleg = WorksheetFunction.Acos(WF_Clamp(cosineDogleg, -1#, 1#))
            normalForce = buoyedWeight * dmd * Abs(dogleg)
            drag = normalForce * frictionFactor
            pooh = CDbl(calcData(r - 1, 7)) + buoyedWeight * dmd + drag
            rih = CDbl(calcData(r - 1, 8)) + buoyedWeight * dmd - drag
        End If
        If r = 1 Then normalForce = 0#: drag = 0#
        slideTorque = surfaceTorque + drag * od / 2#
        rotateTorque = surfaceTorque + drag * od
        backreamTorque = surfaceTorque + drag * od * 1.25
        If dmd > 0# And normalForce > 0# Then
            sinusoidal = 2# * Sqr(youngPa * inertia * normalForce / dmd)
            helical = 4# * Sqr(youngPa * inertia * normalForce / dmd)
        Else
            sinusoidal = 0#: helical = 0#
        End If

        calcData(r, 1) = md: calcData(r, 2) = dmd: calcData(r, 3) = dogleg: calcData(r, 4) = buoyedWeight
        calcData(r, 5) = normalForce: calcData(r, 6) = drag: calcData(r, 7) = pooh: calcData(r, 8) = rih
        calcData(r, 9) = slideTorque: calcData(r, 10) = rotateTorque: calcData(r, 11) = backreamTorque
        calcData(r, 12) = sinusoidal: calcData(r, 13) = helical: calcData(r, 14) = IIf(rih < -helical, "REVIEW", "PASS")

        resultData(r, 1) = md * lengthFactor: resultData(r, 2) = pooh * forceFactor: resultData(r, 3) = rih * forceFactor
        resultData(r, 4) = slideTorque * torqueFactor: resultData(r, 5) = rotateTorque * torqueFactor: resultData(r, 6) = backreamTorque * torqueFactor
        resultData(r, 7) = sinusoidal * forceFactor: resultData(r, 8) = helical * forceFactor: resultData(r, 9) = calcData(r, 14)
        resultData(r, 10) = "": resultData(r, 11) = wsSurvey.Cells(r + 5, 5).Value2

        allData(r, 1) = resultData(r, 1): allData(r, 2) = inc * angleFactor: allData(r, 3) = azi * angleFactor
        allData(r, 4) = md * Cos(inc) * lengthFactor: allData(r, 5) = resultData(r, 2): allData(r, 6) = resultData(r, 3)
        allData(r, 7) = resultData(r, 6): allData(r, 8) = resultData(r, 4): allData(r, 9) = resultData(r, 5)
        allData(r, 10) = resultData(r, 5) * 1.1: allData(r, 11) = resultData(r, 7): allData(r, 12) = resultData(r, 8)
        allData(r, 13) = resultData(r, 9): allData(r, 14) = resultData(r, 11)

        If pooh > peakPooh Then peakPooh = pooh
        If rih < minRih Then minRih = rih: governingDepth = md
        prevMd = md: prevInc = inc: prevAzi = azi
    Next r

    For r = 1 To countRows
        If CDbl(calcData(r, 8)) = minRih Then resultData(r, 10) = "GOVERNING"
    Next r

    wsCalc.Range("A6:N505").ClearContents
    wsResults.Range("A6:K505").ClearContents
    wsAll.Range("A6:N505").ClearContents
    wsCalc.Range("A6").Resize(countRows, 14).Value2 = calcData
    wsResults.Range("A6").Resize(countRows, 11).Value2 = resultData
    wsAll.Range("A6").Resize(countRows, 14).Value2 = allData

    wsSummary.Range("B6").Value2 = peakPooh * forceFactor
    wsSummary.Range("B7").Value2 = minRih * forceFactor
    wsSummary.Range("B8").Value2 = governingDepth * lengthFactor
    wsSummary.Range("A6").Value2 = "Peak POOH hookload " & WF_UnitLabel("Force")
    wsSummary.Range("A7").Value2 = "Lowest RIH axial load " & WF_UnitLabel("Force")
    wsSummary.Range("A8").Value2 = "Governing depth " & WF_UnitLabel("Length")

    wsResults.Range("A5").Value2 = "MD " & WF_UnitLabel("Length")
    wsResults.Range("B5").Value2 = "POOH " & WF_UnitLabel("Force")
    wsResults.Range("C5").Value2 = "RIH " & WF_UnitLabel("Force")
    wsResults.Range("D5:F5").Value2 = WF_Row3("Slide torque " & WF_UnitLabel("Torque"), "Rotate torque " & WF_UnitLabel("Torque"), "Backream torque " & WF_UnitLabel("Torque"))
    wsResults.Range("G5:H5").Value2 = WF_Row2("Sinusoidal limit " & WF_UnitLabel("Force"), "Helical limit " & WF_UnitLabel("Force"))

    wsAll.Range("A5:L5").Value2 = WF_TDAllHeaders()

    WF_WriteTDGraphs wsGraphs, resultData, countRows
    wsGraphs.Range("A3:D3").Value2 = WF_Row4("MD " & WF_UnitLabel("Length"), "POOH " & WF_UnitLabel("Force"), "RIH " & WF_UnitLabel("Force"), "Rotate torque " & WF_UnitLabel("Torque"))
    wsGraphs.Range("E3:F3").Value2 = WF_Row2("MD " & WF_UnitLabel("Length"), "Rotate torque " & WF_UnitLabel("Torque"))
    wsGraphs.Range("A40:D40").Value2 = WF_Row4("MD " & WF_UnitLabel("Length"), "Sinusoidal limit " & WF_UnitLabel("Force"), "Helical limit " & WF_UnitLabel("Force"), "RIH axial " & WF_UnitLabel("Force"))
    WF_ConfigureDepthChart wsGraphs.Name, 1, "Axial load (" & WF_UnitLabel("Force") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureDepthChart wsGraphs.Name, 2, "Torque (" & WF_UnitLabel("Torque") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureDepthChart wsGraphs.Name, 3, "Axial load (" & WF_UnitLabel("Force") & ")", "MD (" & WF_UnitLabel("Length") & ")"

    operationNames = Array("PUW", "SOW", "BKR", "SLD", "ROT", "DRLG")
    axialColumns = Array(5, 6, 6, 6, 6, 6)
    torqueColumns = Array(9, 10, 7, 8, 9, 10)
    wsOps.Range("A5:E500").ClearContents
    helperRow = 5
    For opIndex = 0 To 5
        ReDim opData(1 To countRows, 1 To 8)
        For r = 1 To countRows
            opData(r, 1) = allData(r, 1)
            opData(r, 2) = allData(r, axialColumns(opIndex))
            opData(r, 3) = allData(r, torqueColumns(opIndex))
            opData(r, 4) = allData(r, 11)
            opData(r, 5) = allData(r, 12)
            opData(r, 6) = CDbl(opData(r, 2)) - CDbl(opData(r, 5))
            opData(r, 7) = allData(r, 13)
            opData(r, 8) = allData(r, 14)
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
    WF_WriteTDIndustryDashboard calcData, resultData, allData, countRows, lengthFactor, forceFactor, torqueFactor, angleFactor
End Sub

Public Sub WF_WriteTDIndustryDashboard(ByRef calcData() As Variant, ByRef resultData() As Variant, ByRef allData() As Variant, ByVal rowCount As Long, ByVal lengthFactor As Double, ByVal forceFactor As Double, ByVal torqueFactor As Double, ByVal angleFactor As Double)
    Dim ws As Worksheet, wsObserved As Worksheet, wsSettings As Worksheet
    Dim axialData() As Variant, torqueData() As Variant, inclinationData() As Variant, frictionData() As Variant
    Dim r As Long, sourceRow As Long, nearestRow As Long, selectedMd As Double
    Dim observedHookload As Double, observedTorque As Double, tensionRating As Double, torsionalRating As Double
    Dim neutralLoad As Double, lowFriction As Double, highFriction As Double, showObserved As Boolean, showLimits As Boolean
    Dim reader(1 To 10, 1 To 1) As Variant
    Set ws = ThisWorkbook.Worksheets("Engineering Dashboard")
    Set wsObserved = ThisWorkbook.Worksheets("Observed Data")
    Set wsSettings = ThisWorkbook.Worksheets("Chart Settings")
    ReDim axialData(1 To rowCount, 1 To 11)
    ReDim torqueData(1 To rowCount, 1 To 6)
    ReDim inclinationData(1 To rowCount, 1 To 2)
    ReDim frictionData(1 To rowCount, 1 To 4)
    tensionRating = WF_Num("Inputs", "B13", 1050000#)
    torsionalRating = WF_Num("Inputs", "B14", 62000#)
    lowFriction = WF_Num("Chart Settings", "B8", 0.8)
    highFriction = WF_Num("Chart Settings", "B10", 1.2)
    showObserved = (StrComp(WF_Str("Chart Settings", "B7", "Yes"), "Yes", vbTextCompare) = 0)
    showLimits = (StrComp(WF_Str("Chart Settings", "B12", "Yes"), "Yes", vbTextCompare) = 0)

    For r = 1 To rowCount
        sourceRow = r + 5
        observedHookload = WF_Num("Observed Data", "B" & CStr(sourceRow), CDbl(calcData(r, 7)))
        observedTorque = WF_Num("Observed Data", "C" & CStr(sourceRow), CDbl(calcData(r, 10)))
        axialData(r, 1) = resultData(r, 1)
        axialData(r, 2) = resultData(r, 2)
        axialData(r, 3) = resultData(r, 3)
        axialData(r, 4) = (CDbl(calcData(r, 7)) + WF_Num("Inputs", "B7") * 0.35) * forceFactor
        axialData(r, 5) = (CDbl(calcData(r, 8)) - WF_Num("Inputs", "B7")) * forceFactor
        axialData(r, 6) = (CDbl(calcData(r, 8)) - WF_Num("Inputs", "B7") * 0.25) * forceFactor
        axialData(r, 7) = (CDbl(calcData(r, 8)) - WF_Num("Inputs", "B7") * 0.5) * forceFactor
        If showObserved Then axialData(r, 8) = observedHookload * forceFactor Else axialData(r, 8) = CVErr(xlErrNA)
        If showLimits Then
            axialData(r, 9) = tensionRating * forceFactor
            axialData(r, 10) = resultData(r, 7)
            axialData(r, 11) = resultData(r, 8)
        Else
            axialData(r, 9) = CVErr(xlErrNA): axialData(r, 10) = CVErr(xlErrNA): axialData(r, 11) = CVErr(xlErrNA)
        End If

        torqueData(r, 1) = resultData(r, 1)
        torqueData(r, 2) = resultData(r, 6)
        torqueData(r, 3) = resultData(r, 5)
        torqueData(r, 4) = resultData(r, 5) * 1.1
        If showObserved Then torqueData(r, 5) = observedTorque * torqueFactor Else torqueData(r, 5) = CVErr(xlErrNA)
        If showLimits Then torqueData(r, 6) = torsionalRating * torqueFactor Else torqueData(r, 6) = CVErr(xlErrNA)
        inclinationData(r, 1) = resultData(r, 1)
        inclinationData(r, 2) = CDbl(allData(r, 2))
        neutralLoad = WF_Num("Inputs", "B7") + CDbl(calcData(r, 4)) * CDbl(calcData(r, 1))
        frictionData(r, 1) = resultData(r, 1)
        frictionData(r, 2) = (neutralLoad + (CDbl(calcData(r, 7)) - neutralLoad) * lowFriction) * forceFactor
        frictionData(r, 3) = resultData(r, 2)
        frictionData(r, 4) = (neutralLoad + (CDbl(calcData(r, 7)) - neutralLoad) * highFriction) * forceFactor
    Next r

    ws.Range("A61:Z560").ClearContents
    ws.Range("A61").Resize(rowCount, 11).Value2 = axialData
    ws.Range("M61").Resize(rowCount, 6).Value2 = torqueData
    ws.Range("T61").Resize(rowCount, 2).Value2 = inclinationData
    ws.Range("W61").Resize(rowCount, 4).Value2 = frictionData
    ws.Range("A60:K60").Value2 = WF_TDIndustryAxialHeaders()
    ws.Range("M60:R60").Value2 = WF_TDIndustryTorqueHeaders()
    ws.Range("T60:U60").Value2 = WF_Row2("MD " & WF_UnitLabel("Length"), "Inclination " & WF_UnitLabel("Angle"))
    ws.Range("W60:Z60").Value2 = WF_Row4("MD " & WF_UnitLabel("Length"), "Low friction", "Base friction", "High friction")

    selectedMd = WF_Num("Chart Settings", "B6")
    If selectedMd < CDbl(calcData(1, 1)) Then selectedMd = CDbl(calcData(1, 1))
    If selectedMd > CDbl(calcData(rowCount, 1)) Then selectedMd = CDbl(calcData(rowCount, 1))
    wsSettings.Range("B6").Value2 = selectedMd
    nearestRow = WF_NearestDepthRow("Survey", 6, rowCount, 1, selectedMd) - 5
    ws.Range("B5").Value2 = selectedMd * lengthFactor: ws.Range("C5").Value2 = WF_UnitLabel("Length")
    reader(1, 1) = axialData(nearestRow, 1)
    reader(2, 1) = inclinationData(nearestRow, 2)
    reader(3, 1) = axialData(nearestRow, 2)
    reader(4, 1) = axialData(nearestRow, 3)
    reader(5, 1) = axialData(nearestRow, 8)
    reader(6, 1) = axialData(nearestRow, 9) - axialData(nearestRow, 8)
    reader(7, 1) = torqueData(nearestRow, 3)
    reader(8, 1) = torqueData(nearestRow, 5)
    reader(9, 1) = torqueData(nearestRow, 6) - torqueData(nearestRow, 5)
    reader(10, 1) = IIf(CDbl(reader(6, 1)) < 0# Or CDbl(reader(9, 1)) < 0#, "REVIEW", "WITHIN LIMITS")
    ws.Range("B8:B17").Value2 = reader

    WF_ConfigureDepthChart ws.Name, 1, "Axial load (" & WF_UnitLabel("Force") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureDepthChart ws.Name, 2, "Torque (" & WF_UnitLabel("Torque") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureDepthChart ws.Name, 3, "Inclination (" & WF_UnitLabel("Angle") & ")", "MD (" & WF_UnitLabel("Length") & ")"
    WF_ConfigureDepthChart ws.Name, 4, "Axial load (" & WF_UnitLabel("Force") & ")", "MD (" & WF_UnitLabel("Length") & ")"
End Sub

Private Function WF_TDIndustryAxialHeaders() As Variant
    Dim output(1 To 1, 1 To 11) As Variant
    output(1, 1) = "MD " & WF_UnitLabel("Length")
    output(1, 2) = "PUW": output(1, 3) = "SOW": output(1, 4) = "BKR": output(1, 5) = "SLD": output(1, 6) = "ROT": output(1, 7) = "DRLG"
    output(1, 8) = "Observed hookload": output(1, 9) = "Tension rating": output(1, 10) = "Sinusoidal buckling": output(1, 11) = "Helical buckling"
    WF_TDIndustryAxialHeaders = output
End Function

Private Function WF_TDIndustryTorqueHeaders() As Variant
    Dim output(1 To 1, 1 To 6) As Variant
    output(1, 1) = "MD " & WF_UnitLabel("Length"): output(1, 2) = "BKR": output(1, 3) = "ROT": output(1, 4) = "DRLG"
    output(1, 5) = "Observed torque": output(1, 6) = "Torsional rating"
    WF_TDIndustryTorqueHeaders = output
End Function

Private Function WF_ContiguousRows(ByVal ws As Worksheet, ByVal firstRow As Long, ByVal keyColumn As Long, ByVal maximumRows As Long) As Long
    Dim r As Long
    For r = 0 To maximumRows - 1
        If Len(Trim$(CStr(ws.Cells(firstRow + r, keyColumn).Value2))) = 0 Then Exit For
        WF_ContiguousRows = WF_ContiguousRows + 1
    Next r
End Function

Public Function WF_TDAllHeaders() As Variant
    Dim output(1 To 1, 1 To 12) As Variant
    output(1, 1) = "MD " & WF_UnitLabel("Length"): output(1, 2) = "Inc " & WF_UnitLabel("Angle"): output(1, 3) = "Azi " & WF_UnitLabel("Angle")
    output(1, 4) = "TVD screen " & WF_UnitLabel("Length"): output(1, 5) = "PUW axial " & WF_UnitLabel("Force"): output(1, 6) = "SOW axial " & WF_UnitLabel("Force")
    output(1, 7) = "BKR torque " & WF_UnitLabel("Torque"): output(1, 8) = "SLD torque " & WF_UnitLabel("Torque"): output(1, 9) = "ROT torque " & WF_UnitLabel("Torque")
    output(1, 10) = "DRLG torque " & WF_UnitLabel("Torque"): output(1, 11) = "Sinusoidal " & WF_UnitLabel("Force"): output(1, 12) = "Helical " & WF_UnitLabel("Force")
    WF_TDAllHeaders = output
End Function

Public Sub WF_WriteTDGraphs(ByVal ws As Worksheet, ByRef results() As Variant, ByVal rowCount As Long)
    Dim firstBlock() As Variant, torqueBlock() As Variant, bucklingBlock() As Variant, r As Long
    ReDim firstBlock(1 To rowCount, 1 To 4): ReDim torqueBlock(1 To rowCount, 1 To 2): ReDim bucklingBlock(1 To rowCount, 1 To 4)
    ws.Range("A4:F505").ClearContents: ws.Range("A41:D540").ClearContents
    For r = 1 To rowCount
        firstBlock(r, 1) = results(r, 1): firstBlock(r, 2) = results(r, 2): firstBlock(r, 3) = results(r, 3): firstBlock(r, 4) = results(r, 5)
        torqueBlock(r, 1) = results(r, 1): torqueBlock(r, 2) = results(r, 5)
        bucklingBlock(r, 1) = results(r, 1): bucklingBlock(r, 2) = results(r, 7): bucklingBlock(r, 3) = results(r, 8): bucklingBlock(r, 4) = results(r, 3)
    Next r
    ws.Range("A4").Resize(rowCount, 4).Value2 = firstBlock
    ws.Range("E4").Resize(rowCount, 2).Value2 = torqueBlock
    ws.Range("A41").Resize(rowCount, 4).Value2 = bucklingBlock
End Sub

Public Function WF_TDForceHelper(ByRef source() As Variant, ByVal rowCount As Long) As Variant
    Dim output() As Variant, r As Long: ReDim output(1 To rowCount, 1 To 4)
    For r = 1 To rowCount: output(r, 1) = source(r, 1): output(r, 2) = source(r, 2): output(r, 3) = source(r, 4): output(r, 4) = source(r, 5): Next r
    WF_TDForceHelper = output
End Function

Public Function WF_TDTorqueHelper(ByRef source() As Variant, ByVal rowCount As Long) As Variant
    Dim output() As Variant, r As Long: ReDim output(1 To rowCount, 1 To 2)
    For r = 1 To rowCount: output(r, 1) = source(r, 1): output(r, 2) = source(r, 3): Next r
    WF_TDTorqueHelper = output
End Function

Public Function WF_TDOverviewHelper(ByRef source() As Variant, ByVal rowCount As Long) As Variant
    Dim output() As Variant, r As Long: ReDim output(1 To rowCount, 1 To 5)
    For r = 1 To rowCount: output(r, 1) = source(r, 1): output(r, 2) = source(r, 2): output(r, 3) = source(r, 1): output(r, 4) = source(r, 3): output(r, 5) = source(r, 6): Next r
    WF_TDOverviewHelper = output
End Function

Public Function WF_Row2(ByVal a As Variant, ByVal b As Variant) As Variant
    Dim output(1 To 1, 1 To 2) As Variant: output(1, 1) = a: output(1, 2) = b: WF_Row2 = output
End Function

Public Function WF_Row3(ByVal a As Variant, ByVal b As Variant, ByVal c As Variant) As Variant
    Dim output(1 To 1, 1 To 3) As Variant: output(1, 1) = a: output(1, 2) = b: output(1, 3) = c: WF_Row3 = output
End Function

Public Function WF_Row4(ByVal a As Variant, ByVal b As Variant, ByVal c As Variant, ByVal d As Variant) As Variant
    Dim output(1 To 1, 1 To 4) As Variant: output(1, 1) = a: output(1, 2) = b: output(1, 3) = c: output(1, 4) = d: WF_Row4 = output
End Function

Public Function WF_Row5(ByVal a As Variant, ByVal b As Variant, ByVal c As Variant, ByVal d As Variant, ByVal e As Variant) As Variant
    Dim output(1 To 1, 1 To 5) As Variant: output(1, 1) = a: output(1, 2) = b: output(1, 3) = c: output(1, 4) = d: output(1, 5) = e: WF_Row5 = output
End Function
