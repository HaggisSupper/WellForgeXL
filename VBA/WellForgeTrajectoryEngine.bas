Attribute VB_Name = "WellForgeTrajectoryEngine"
Option Explicit

Private Const WF_TRAJECTORY_TIMEOUT_SECONDS As Double = 120#
Private Const WF_TRAJECTORY_EXECUTION_MODE As String = "RUST REQUIRED — NO VBA FALLBACK"
Private Const WF_TRAJECTORY_MAX_PLAN = 500
Private Const WF_TRAJECTORY_MAX_SURVEY = 500
Private Const WF_TRAJECTORY_MAX_TARGETS = 100
Private Const WF_TRAJECTORY_MAX_SLIDES = 200
Private Const WF_TRAJECTORY_MAX_FORMATIONS = 100
Private Const WF_TRAJECTORY_PI As Double = 3.14159265358979
Private Const WF_TRAJECTORY_TWO_PI As Double = 6.28318530717959

Private WF_TRAJECTORY_INJECT_COMMIT_FAILURE As Boolean
Private WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED As Boolean

Private Type WF_TrajectorySystemTime
    Year As Integer
    Month As Integer
    DayOfWeek As Integer
    Day As Integer
    Hour As Integer
    Minute As Integer
    Second As Integer
    Milliseconds As Integer
End Type

#If VBA7 Then
Private Declare PtrSafe Sub GetSystemTime Lib "kernel32" (ByRef value As WF_TrajectorySystemTime)
#Else
Private Declare Sub GetSystemTime Lib "kernel32" (ByRef value As WF_TrajectorySystemTime)
#End If

Private Type WF_TrajectoryBridgeStage
    AnalysisId As String
    RequestHash As String
    ResultHash As String
    EngineVersion As String
    ResultStatus As String
    PlanCount As Long
    SurveyCount As Long
    TargetCount As Long
    SlideCount As Long
    FormationCount As Long
    ProjectionCount As Long
    PlanRecords As Variant
    SurveyRecords As Variant
    ResidualRecords As Variant
    TargetRecords As Variant
    SlideRecords As Variant
    FormationRecords As Variant
    ProjectionRecord As Variant
End Type

Public Sub WF_RunTrajectoryRustEngine()
    Dim executablePath As String, hashManifestPath As String, runPath As String
    Dim requestPath As String, resultPath As String, diagnosticPath As String, bridgePath As String
    Dim expectedEngineHash As String, actualEngineHash As String, requestHash As String
    Dim exitCode As Long, stdOutText As String, stdErrText As String
    Dim request As Object, expectedIds As Object, staged As WF_TrajectoryBridgeStage
    Dim failureState As String, failureNumber As Long, failureDescription As String
    Dim lastAcceptedValuesPreserved As Boolean, previousBusy As Boolean, previousEvents As Boolean, previousInteractive As Boolean
    Dim runtimeCaptured As Boolean
    On Error GoTo Failed

    previousBusy = WF_Busy
    previousEvents = Application.EnableEvents
    previousInteractive = Application.Interactive
    runtimeCaptured = True
    WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED = False
    WF_Busy = True
    Application.EnableEvents = False
    Application.Interactive = False
    failureState = "ENGINE UNAVAILABLE"
    runPath = WF_CreateFreshTrajectoryRunDirectory()
    requestPath = runPath & Application.PathSeparator & "request.json"
    resultPath = runPath & Application.PathSeparator & "result.json"
    diagnosticPath = runPath & Application.PathSeparator & "diagnostics.jsonl"
    bridgePath = runPath & Application.PathSeparator & "result.wfbridge"

    executablePath = ThisWorkbook.Path & Application.PathSeparator & "wellforge-trajectory.exe"
    hashManifestPath = ThisWorkbook.Path & Application.PathSeparator & "wellforge-trajectory.exe.sha256"
    failureState = "ENGINE UNAVAILABLE"
    If Len(Dir$(executablePath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8800, "WF_RunTrajectoryRustEngine", "ENGINE UNAVAILABLE: " & executablePath
    If Len(Dir$(hashManifestPath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8801, "WF_RunTrajectoryRustEngine", "ENGINE UNAVAILABLE: " & hashManifestPath

    failureState = "ENGINE HASH MISMATCH"
    expectedEngineHash = LCase$(Trim$(ReadUtf8File(hashManifestPath)))
    If Not WF_TrajectoryIsSha256(expectedEngineHash) Then Err.Raise vbObjectError + 8802, "WF_RunTrajectoryRustEngine", "ENGINE HASH MISMATCH: invalid wellforge-trajectory.exe.sha256"
    actualEngineHash = WF_TrajectoryFileSha256(executablePath)
    If StrComp(actualEngineHash, expectedEngineHash, vbBinaryCompare) <> 0 Then Err.Raise vbObjectError + 8803, "WF_RunTrajectoryRustEngine", "ENGINE HASH MISMATCH"

    failureState = "INVALID REQUEST"
    Set expectedIds = WF_TrajectoryExpectedIds()
    Set request = WF_BuildTrajectoryRequest()
    AtomicWriteUtf8 requestPath, JsonStringify(request, 0)
    exitCode = WF_ExecTrajectoryBounded(WF_TrajectoryQuote(executablePath) & " validate --input " & WF_TrajectoryQuote(requestPath), WF_TRAJECTORY_TIMEOUT_SECONDS, stdOutText, stdErrText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8804, "WF_RunTrajectoryRustEngine", "INVALID REQUEST: " & Trim$(stdErrText)

    failureState = "ANALYSIS FAILED"
    exitCode = WF_ExecTrajectoryBounded(WF_TrajectoryQuote(executablePath) & " run --input " & WF_TrajectoryQuote(requestPath) & " --output " & WF_TrajectoryQuote(resultPath) & " --diagnostics " & WF_TrajectoryQuote(diagnosticPath) & " --no-backup", WF_TRAJECTORY_TIMEOUT_SECONDS, stdOutText, stdErrText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8805, "WF_RunTrajectoryRustEngine", "ANALYSIS FAILED: " & Trim$(stdErrText)
    If Len(Dir$(resultPath, vbNormal)) = 0 Or Len(Dir$(diagnosticPath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8806, "WF_RunTrajectoryRustEngine", "ANALYSIS FAILED: result or diagnostics file missing"
    failureState = "INVALID RESULT"
    requestHash = WF_TrajectoryRequestHashFromDiagnostics(diagnosticPath)

    exitCode = WF_ExecTrajectoryBounded(WF_TrajectoryQuote(executablePath) & " verify-result --input " & WF_TrajectoryQuote(resultPath) & " --request-hash " & WF_TrajectoryQuote(requestHash), WF_TRAJECTORY_TIMEOUT_SECONDS, stdOutText, stdErrText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8807, "WF_RunTrajectoryRustEngine", "INVALID RESULT: " & Trim$(stdErrText)
    exitCode = WF_ExecTrajectoryBounded(WF_TrajectoryQuote(executablePath) & " bridge --input " & WF_TrajectoryQuote(resultPath) & " --output " & WF_TrajectoryQuote(bridgePath) & " --request-hash " & WF_TrajectoryQuote(requestHash), WF_TRAJECTORY_TIMEOUT_SECONDS, stdOutText, stdErrText)
    If exitCode <> 0 Or Len(Dir$(bridgePath, vbNormal)) = 0 Then Err.Raise vbObjectError + 8808, "WF_RunTrajectoryRustEngine", "INVALID RESULT: bridge generation failed: " & Trim$(stdErrText)

    WF_ParseAndValidateTrajectoryBridge ReadUtf8File(bridgePath), requestHash, CStr(ThisWorkbook.Worksheets("Inputs").Range("Q12").Value2), expectedIds, staged
    WF_CommitTrajectoryBridge staged, requestPath, resultPath, diagnosticPath, actualEngineHash, WF_TrajectoryUtcTimestamp()
    Application.Interactive = previousInteractive
    Application.EnableEvents = previousEvents
    WF_Busy = previousBusy
    Exit Sub

Failed:
    failureNumber = Err.Number
    failureDescription = Err.Description
    lastAcceptedValuesPreserved = (InStr(1, failureDescription, "ROLLBACK INCOMPLETE", vbTextCompare) = 0)
    On Error Resume Next
    WF_PublishTrajectoryFailure failureState, diagnosticPath, lastAcceptedValuesPreserved
    If runtimeCaptured Then
        Application.Interactive = previousInteractive
        Application.EnableEvents = previousEvents
        WF_Busy = previousBusy
    End If
    On Error GoTo 0
    Err.Raise failureNumber, "WF_RunTrajectoryRustEngine", failureDescription
End Sub

Public Sub WellForge_TrajectoryRollbackSelfTest()
    Dim failureNumber As Long, failureDescription As String, injectedFailure As Long
    Dim telemetryDiagnostic As String
    Dim snapshots As Collection
    On Error GoTo Failed

    WF_RunTrajectoryRustEngine
    Set snapshots = WF_TrajectoryCaptureSnapshots()
    WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED = False
    WF_TRAJECTORY_INJECT_COMMIT_FAILURE = True
    On Error Resume Next
    WF_RunTrajectoryRustEngine
    injectedFailure = Err.Number
    Err.Clear
    On Error GoTo Failed
    WF_TRAJECTORY_INJECT_COMMIT_FAILURE = False
    If injectedFailure = 0 Then Err.Raise vbObjectError + 8900, "WellForge_TrajectoryRollbackSelfTest", "INJECTED COMMIT FAILURE DID NOT OCCUR"
    If Not WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED Then Err.Raise vbObjectError + 8901, "WellForge_TrajectoryRollbackSelfTest", "ROLLBACK EQUALITY VERIFICATION FAILED"
    If Not WF_TrajectorySnapshotsMatch(snapshots) Then Err.Raise vbObjectError + 8903, "WellForge_TrajectoryRollbackSelfTest", "END-TO-END ROLLBACK EQUALITY VERIFICATION FAILED"
    If StrComp(CStr(ThisWorkbook.Worksheets("Results").Range("P5").Value2), WF_TRAJECTORY_EXECUTION_MODE, vbBinaryCompare) <> 0 Then Err.Raise vbObjectError + 8904, "WellForge_TrajectoryRollbackSelfTest", "FAILURE MODE TELEMETRY VERIFICATION FAILED"
    If StrComp(CStr(ThisWorkbook.Worksheets("Results").Range("P6").Value2), "INVALID RESULT — FAILED — LAST ACCEPTED VALUES PRESERVED", vbBinaryCompare) <> 0 Then Err.Raise vbObjectError + 8905, "WellForge_TrajectoryRollbackSelfTest", "FAILURE STATUS TELEMETRY VERIFICATION FAILED"
    telemetryDiagnostic = CStr(ThisWorkbook.Worksheets("Results").Range("P9").Value2)
    If Len(telemetryDiagnostic) = 0 Or Len(Dir$(telemetryDiagnostic, vbNormal)) = 0 Then Err.Raise vbObjectError + 8906, "WellForge_TrajectoryRollbackSelfTest", "FAILURE DIAGNOSTIC TELEMETRY VERIFICATION FAILED"
    WF_RunTrajectoryRustEngine
    Exit Sub

Failed:
    failureNumber = Err.Number
    failureDescription = Err.Description
    WF_TRAJECTORY_INJECT_COMMIT_FAILURE = False
    Err.Raise failureNumber, "WellForge_TrajectoryRollbackSelfTest", failureDescription
End Sub

Private Function WF_BuildTrajectoryRequest() As Object
    Dim root As Object, sources As Object, mdDatum As Object, projection As Object
    Dim provenance As Variant, metadata As Variant, identities As Variant
    Dim plan As Collection, survey As Collection, targets As Collection, slides As Collection, formations As Collection
    WF_TrajectoryValidateUnitAndControlInputs
    provenance = ThisWorkbook.Worksheets("Inputs").Range("Q6:V9").Value2
    metadata = ThisWorkbook.Worksheets("Inputs").Range("Q12:Q17").Value2
    identities = ThisWorkbook.Worksheets("Calc").Range("JA7:JE506").Value2

    Set root = CreateObject("Scripting.Dictionary")
    root.Add "contract_version", WF_TrajectoryRequiredText(metadata(6, 1), "contract_version")
    root.Add "analysis_id", WF_TrajectoryRequiredText(metadata(1, 1), "analysis_id")
    Set sources = CreateObject("Scripting.Dictionary")
    sources.Add "well", WF_TrajectorySourceFromRow(provenance, 1)
    sources.Add "wellbore", WF_TrajectorySourceFromRow(provenance, 2)
    sources.Add "plan_trajectory", WF_TrajectorySourceFromRow(provenance, 3)
    sources.Add "survey_trajectory", WF_TrajectorySourceFromRow(provenance, 4)
    root.Add "sources", sources

    Set mdDatum = CreateObject("Scripting.Dictionary")
    mdDatum.Add "uid", WF_TrajectoryRequiredText(metadata(2, 1), "md_datum.uid")
    mdDatum.Add "name", WF_TrajectoryRequiredText(metadata(3, 1), "md_datum.name")
    mdDatum.Add "kind", WF_TrajectoryRequiredText(metadata(4, 1), "md_datum.kind")
    root.Add "md_datum", mdDatum
    root.Add "azimuth_reference", WF_TrajectoryRequiredText(metadata(5, 1), "azimuth_reference")
    root.Add "vertical_section_azimuth_rad", WF_Mod2Pi(WF_ToSI(WF_TrajectoryInputNumber("B16", "vertical section azimuth"), WF_Str("Inputs", "E6", "rad")))

    Set plan = WF_TrajectoryStations("Plan", 2, "E5", "E6", identities, 1, "plan")
    Set survey = WF_TrajectoryStations("Survey", 2, "E7", "E8", identities, 2, "survey")
    Set targets = WF_TrajectoryTargets(identities)
    Set slides = WF_TrajectorySlides(identities)
    Set formations = WF_TrajectoryFormations(identities)
    root.Add "plan", plan
    root.Add "survey", survey
    root.Add "targets", targets
    root.Add "slides", slides
    root.Add "formations", formations
    If WF_TrajectoryProjectionRequested() Then
        Set projection = WF_TrajectoryProjection()
        root.Add "projection", projection
    Else
        root.Add "projection", Empty
    End If
    Set WF_BuildTrajectoryRequest = root
End Function

Private Function WF_TrajectorySourceFromRow(ByVal provenance As Variant, ByVal rowIndex As Long) As Object
    Dim source As Object
    Set source = CreateObject("Scripting.Dictionary")
    source.Add "uuid", WF_TrajectoryRequiredText(provenance(rowIndex, 1), "sources.uuid")
    source.Add "uri", WF_TrajectoryRequiredText(provenance(rowIndex, 2), "sources.uri")
    source.Add "object_type", WF_TrajectoryRequiredText(provenance(rowIndex, 3), "sources.object_type")
    source.Add "content_hash", WF_TrajectoryRequiredText(provenance(rowIndex, 4), "sources.content_hash")
    source.Add "citation_name", WF_TrajectoryRequiredText(provenance(rowIndex, 5), "sources.citation_name")
    source.Add "source_system", WF_TrajectoryRequiredText(provenance(rowIndex, 6), "sources.source_system")
    Set WF_TrajectorySourceFromRow = source
End Function

Private Function WF_TrajectoryStations(ByVal sheetName As String, ByVal keyColumn As Long, ByVal lengthUnitCell As String, ByVal angleUnitCell As String, ByVal identities As Variant, ByVal identityColumn As Long, ByVal laterKind As String) As Collection
    Dim output As Collection, station As Object, ws As Worksheet
    Dim countRows As Long, slot As Long, worksheetRow As Long, kind As String
    Set output = New Collection
    Set ws = ThisWorkbook.Worksheets(sheetName)
    countRows = WF_TrajectoryContiguousCount(sheetName, keyColumn, WF_TRAJECTORY_MAX_PLAN)
    For slot = 1 To countRows
        worksheetRow = slot + 6
        If slot = 1 Then kind = "tie_in" Else kind = laterKind
        Set station = CreateObject("Scripting.Dictionary")
        station.Add "uid", WF_TrajectoryRequiredText(identities(slot, identityColumn), sheetName & " station uid")
        station.Add "kind", kind
        station.Add "md_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 2).Value2, sheetName & " MD"), WF_Str("Inputs", lengthUnitCell, "m"))
        station.Add "inclination_rad", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 3).Value2, sheetName & " inclination"), WF_Str("Inputs", angleUnitCell, "rad"))
        station.Add "azimuth_rad", WF_Mod2Pi(WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 4).Value2, sheetName & " azimuth"), WF_Str("Inputs", angleUnitCell, "rad")))
        output.Add station
    Next slot
    Set WF_TrajectoryStations = output
End Function

Private Function WF_TrajectoryTargets(ByVal identities As Variant) As Collection
    Dim output As Collection, target As Object, ws As Worksheet
    Dim slot As Long, worksheetRow As Long, countRows As Long, surfaceNorth As Double, surfaceEast As Double
    Set output = New Collection
    Set ws = ThisWorkbook.Worksheets("Targets")
    countRows = WF_TrajectoryContiguousCount("Targets", 1, WF_TRAJECTORY_MAX_TARGETS)
    surfaceNorth = WF_ToSI(WF_TrajectoryInputNumber("B13", "surface north"), WF_Str("Inputs", "E5", "m"))
    surfaceEast = WF_ToSI(WF_TrajectoryInputNumber("B14", "surface east"), WF_Str("Inputs", "E5", "m"))
    For slot = 1 To countRows
        worksheetRow = slot + 6
        Set target = CreateObject("Scripting.Dictionary")
        target.Add "uid", WF_TrajectoryRequiredText(identities(slot, 3), "target uid")
        target.Add "name", WF_TrajectoryRequiredText(ws.Cells(worksheetRow, 1).Value2, "target name")
        target.Add "kind", LCase$(Trim$(CStr(ws.Cells(worksheetRow, 6).Value2)))
        target.Add "md_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 2).Value2, "target MD"), WF_Str("Inputs", "E9", "m"))
        target.Add "north_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 3).Value2, "target north"), WF_Str("Inputs", "E9", "m")) - surfaceNorth
        target.Add "east_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 4).Value2, "target east"), WF_Str("Inputs", "E9", "m")) - surfaceEast
        target.Add "tvd_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 5).Value2, "target TVD"), WF_Str("Inputs", "E9", "m"))
        target.Add "major_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 7).Value2, "target major"), WF_Str("Inputs", "E9", "m"))
        target.Add "minor_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 8).Value2, "target minor"), WF_Str("Inputs", "E9", "m"))
        target.Add "rotation_rad", WF_Mod2Pi(WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 9).Value2, "target rotation"), WF_Str("Inputs", "E8", "rad")))
        target.Add "vertical_tolerance_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 10).Value2, "target vertical tolerance"), WF_Str("Inputs", "E9", "m"))
        output.Add target
    Next slot
    Set WF_TrajectoryTargets = output
End Function

Private Function WF_TrajectorySlides(ByVal identities As Variant) As Collection
    Dim output As Collection, slide As Object, ws As Worksheet
    Dim slot As Long, worksheetRow As Long, countRows As Long
    Set output = New Collection
    Set ws = ThisWorkbook.Worksheets("Slide Performance")
    countRows = WF_TrajectoryContiguousCount("Slide Performance", 1, WF_TRAJECTORY_MAX_SLIDES)
    For slot = 1 To countRows
        worksheetRow = slot + 6
        Set slide = CreateObject("Scripting.Dictionary")
        slide.Add "uid", WF_TrajectoryRequiredText(identities(slot, 4), "slide uid")
        slide.Add "md_in_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 3).Value2, "slide MD in"), WF_Str("Inputs", "E10", "m"))
        slide.Add "md_out_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 4).Value2, "slide MD out"), WF_Str("Inputs", "E10", "m"))
        slide.Add "slide_length_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 5).Value2, "slide length"), WF_Str("Inputs", "E10", "m"))
        slide.Add "commanded_toolface_rad", WF_Mod2Pi(WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 7).Value2, "commanded toolface"), WF_Str("Inputs", "E8", "rad")))
        slide.Add "rotary_build_rad_per_m", WF_TrajectoryOptionalInputNumber(ws.Cells(worksheetRow, 8).Value2, WF_Str("Inputs", "K9", "rad/m"))
        slide.Add "rotary_effective_turn_rad_per_m", WF_TrajectoryOptionalInputNumber(ws.Cells(worksheetRow, 9).Value2, WF_Str("Inputs", "K9", "rad/m"))
        slide.Add "low_inclination_threshold_rad", WF_ToSI(WF_TrajectoryRequiredNumber(ThisWorkbook.Worksheets("Inputs").Range("N5").Value2, "low inclination threshold"), "deg")
        output.Add slide
    Next slot
    Set WF_TrajectorySlides = output
End Function

Private Function WF_TrajectoryFormations(ByVal identities As Variant) As Collection
    Dim output As Collection, formation As Object, ws As Worksheet
    Dim slot As Long, worksheetRow As Long, countRows As Long, optionalValue As Variant
    Set output = New Collection
    Set ws = ThisWorkbook.Worksheets("Formation Tops")
    countRows = WF_TrajectoryContiguousCount("Formation Tops", 1, WF_TRAJECTORY_MAX_FORMATIONS)
    For slot = 1 To countRows
        worksheetRow = slot + 6
        Set formation = CreateObject("Scripting.Dictionary")
        formation.Add "uid", WF_TrajectoryRequiredText(identities(slot, 5), "formation uid")
        formation.Add "name", WF_TrajectoryRequiredText(ws.Cells(worksheetRow, 1).Value2, "formation name")
        formation.Add "prognosed_tvd_m", WF_ToSI(WF_TrajectoryRequiredNumber(ws.Cells(worksheetRow, 3).Value2, "formation prognosed TVD"), WF_Str("Inputs", "E11", "m"))
        optionalValue = WF_TrajectoryOptionalNullableNumber(ws.Cells(worksheetRow, 4).Value2, WF_Str("Inputs", "E11", "m"), "formation actual MD")
        formation.Add "actual_md_m", optionalValue
        optionalValue = WF_TrajectoryOptionalNullableNumber(ws.Cells(worksheetRow, 5).Value2, WF_Str("Inputs", "E11", "m"), "formation tolerance")
        formation.Add "tolerance_m", optionalValue
        output.Add formation
    Next slot
    Set WF_TrajectoryFormations = output
End Function

Private Function WF_TrajectoryProjection() As Object
    Dim projection As Object
    Set projection = CreateObject("Scripting.Dictionary")
    projection.Add "bit_md_m", WF_ToSI(WF_TrajectoryRequiredNumber(ThisWorkbook.Worksheets("Inputs").Range("K5").Value2, "projection bit MD"), WF_Str("Inputs", "E7", "m"))
    projection.Add "ahead_m", WF_ToSI(WF_TrajectoryRequiredNumber(ThisWorkbook.Worksheets("Inputs").Range("K6").Value2, "projection ahead"), WF_Str("Inputs", "E7", "m"))
    projection.Add "build_tendency_rad_per_m", WF_ToSI(WF_TrajectoryRequiredNumber(ThisWorkbook.Worksheets("Inputs").Range("K7").Value2, "projection build tendency"), WF_Str("Inputs", "K9", "rad/m"))
    projection.Add "effective_turn_tendency_rad_per_m", WF_ToSI(WF_TrajectoryRequiredNumber(ThisWorkbook.Worksheets("Inputs").Range("K8").Value2, "projection effective turn tendency"), WF_Str("Inputs", "K9", "rad/m"))
    projection.Add "low_inclination_threshold_rad", WF_ToSI(WF_TrajectoryRequiredNumber(ThisWorkbook.Worksheets("Inputs").Range("N5").Value2, "projection low inclination threshold"), "deg")
    Set WF_TrajectoryProjection = projection
End Function

Private Function WF_TrajectoryProjectionRequested() As Boolean
    Dim values As Variant, index As Long, populated As Long
    values = ThisWorkbook.Worksheets("Inputs").Range("K5:K8").Value2
    For index = 1 To 4
        If Len(Trim$(CStr(values(index, 1)))) > 0 Then populated = populated + 1
    Next index
    If populated <> 0 And populated <> 4 Then Err.Raise vbObjectError + 8810, "WF_TrajectoryProjectionRequested", "INVALID REQUEST: projection K5:K8 must be completely populated or completely blank"
    WF_TrajectoryProjectionRequested = (populated = 4)
End Function

Private Function WF_TrajectoryExpectedIds() As Object
    Dim output As Object, identities As Variant, group As Object
    Set output = CreateObject("Scripting.Dictionary")
    identities = ThisWorkbook.Worksheets("Calc").Range("JA7:JE506").Value2
    Set group = WF_TrajectoryExpectedGroup(identities, 1, WF_TrajectoryContiguousCount("Plan", 2, WF_TRAJECTORY_MAX_PLAN), "plan")
    output.Add "P", group
    Set group = WF_TrajectoryExpectedGroup(identities, 2, WF_TrajectoryContiguousCount("Survey", 2, WF_TRAJECTORY_MAX_SURVEY), "survey")
    output.Add "S", group
    Set group = WF_TrajectoryExpectedGroup(identities, 3, WF_TrajectoryContiguousCount("Targets", 1, WF_TRAJECTORY_MAX_TARGETS), "target")
    output.Add "T", group
    Set group = WF_TrajectoryExpectedGroup(identities, 4, WF_TrajectoryContiguousCount("Slide Performance", 1, WF_TRAJECTORY_MAX_SLIDES), "slide")
    output.Add "L", group
    Set group = WF_TrajectoryExpectedGroup(identities, 5, WF_TrajectoryContiguousCount("Formation Tops", 1, WF_TRAJECTORY_MAX_FORMATIONS), "formation")
    output.Add "F", group
    output.Add "X", IIf(WF_TrajectoryProjectionRequested(), 1, 0)
    Set WF_TrajectoryExpectedIds = output
End Function

Private Function WF_TrajectoryExpectedGroup(ByVal identities As Variant, ByVal identityColumn As Long, ByVal countRows As Long, ByVal groupName As String) As Object
    Dim output As Object, slot As Long, identity As String
    Set output = CreateObject("Scripting.Dictionary")
    output.CompareMode = vbTextCompare
    For slot = 1 To countRows
        identity = LCase$(WF_TrajectoryRequiredText(identities(slot, identityColumn), groupName & " uid"))
        If Not WF_TrajectoryIsUuid(identity) Then Err.Raise vbObjectError + 8811, "WF_TrajectoryExpectedGroup", "INVALID REQUEST: invalid " & groupName & " UUID at slot " & CStr(slot)
        If output.Exists(identity) Then Err.Raise vbObjectError + 8812, "WF_TrajectoryExpectedGroup", "INVALID REQUEST: duplicate " & groupName & " UUID"
        output.Add identity, slot
    Next slot
    Set WF_TrajectoryExpectedGroup = output
End Function

Private Function WF_TrajectoryContiguousCount(ByVal sheetName As String, ByVal keyColumn As Long, ByVal maximum As Long) As Long
    Dim ws As Worksheet, slot As Long, populated As Boolean, gapSeen As Boolean, lastPopulatedRow As Long
    Set ws = ThisWorkbook.Worksheets(sheetName)
    lastPopulatedRow = ws.Cells(ws.Rows.Count, keyColumn).End(xlUp).Row
    If lastPopulatedRow > maximum + 6 Then Err.Raise vbObjectError + 8813, "WF_TrajectoryContiguousCount", "INVALID REQUEST: BRIDGE CAPACITY EXCEEDED for " & sheetName
    For slot = 1 To maximum
        populated = Len(Trim$(CStr(ws.Cells(slot + 6, keyColumn).Value2))) > 0
        If populated And gapSeen Then Err.Raise vbObjectError + 8813, "WF_TrajectoryContiguousCount", "INVALID REQUEST: " & sheetName & " contains a gap before row " & CStr(slot + 6)
        If populated Then WF_TrajectoryContiguousCount = slot Else gapSeen = True
    Next slot
End Function

Private Function WF_CreateFreshTrajectoryRunDirectory() As String
    Dim rootPath As String, candidate As String, runId As String, attempt As Long
    rootPath = Environ$("TEMP") & Application.PathSeparator & "WellForgeTrajectory"
    WF_TrajectoryEnsureFolder rootPath
    runId = Format$(Now, "yyyymmdd-hhnnss") & "-" & Format$(CLng((Timer - Fix(Timer)) * 1000#), "000")
    For attempt = 0 To 9999
        candidate = rootPath & Application.PathSeparator & runId & "-" & Format$(attempt, "0000")
        If Len(Dir$(candidate, vbDirectory)) = 0 Then
            MkDir candidate
            WF_CreateFreshTrajectoryRunDirectory = candidate
            Exit Function
        End If
    Next attempt
    Err.Raise vbObjectError + 8814, "WF_CreateFreshTrajectoryRunDirectory", "ENGINE UNAVAILABLE: unable to allocate a fresh run directory"
End Function

Private Function WF_ExecTrajectoryBounded(ByVal commandLine As String, ByVal timeoutSeconds As Double, ByRef stdOutText As String, ByRef stdErrText As String) As Long
    Dim shell As Object, process As Object, started As Double
    Set shell = CreateObject("WScript.Shell")
    Set process = shell.Exec(commandLine)
    started = Timer
    Do While process.Status = 0
        DoEvents
        If WF_TrajectoryElapsedSeconds(started) > timeoutSeconds Then
            process.Terminate
            Err.Raise vbObjectError + 8815, "WF_ExecTrajectoryBounded", "ENGINE TIMEOUT"
        End If
    Loop
    stdOutText = process.StdOut.ReadAll
    stdErrText = process.StdErr.ReadAll
    WF_ExecTrajectoryBounded = process.ExitCode
End Function

Private Function WF_TrajectoryFileSha256(ByVal filePath As String) As String
    Dim scriptText As String, commandLine As String, outputText As String, errorText As String, exitCode As Long
    scriptText = "$p='" & Replace$(filePath, "'", "''") & "'; $h=[System.Security.Cryptography.SHA256]::Create(); try {[BitConverter]::ToString($h.ComputeHash([IO.File]::ReadAllBytes($p))).Replace('-','').ToLowerInvariant()} finally {$h.Dispose()}"
    commandLine = WF_TrajectoryQuote("powershell.exe") & " -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command " & WF_TrajectoryQuote(scriptText)
    exitCode = WF_ExecTrajectoryBounded(commandLine, 30#, outputText, errorText)
    If exitCode <> 0 Then Err.Raise vbObjectError + 8816, "WF_TrajectoryFileSha256", "ENGINE HASH MISMATCH: unable to hash executable: " & Trim$(errorText)
    WF_TrajectoryFileSha256 = LCase$(Trim$(outputText))
    If Not WF_TrajectoryIsSha256(WF_TrajectoryFileSha256) Then Err.Raise vbObjectError + 8817, "WF_TrajectoryFileSha256", "ENGINE HASH MISMATCH: invalid hash output"
End Function

Private Function WF_TrajectoryRequestHashFromDiagnostics(ByVal diagnosticPath As String) As String
    Dim diagnosticText As String, lines As Variant, line As Variant
    Dim marker As String, startAt As Long, foundHash As String
    marker = """request_hash"":"""
    diagnosticText = ReadUtf8File(diagnosticPath)
    lines = Split(Replace$(diagnosticText, vbCr, vbNullString), vbLf)
    For Each line In lines
        If Len(CStr(line)) > 0 Then
            startAt = InStr(1, CStr(line), marker, vbBinaryCompare)
            If startAt = 0 Then Err.Raise vbObjectError + 8818, "WF_TrajectoryRequestHashFromDiagnostics", "INVALID RESULT: diagnostics omit request_hash"
            startAt = startAt + Len(marker)
            foundHash = Mid$(CStr(line), startAt, 64)
            If Not WF_TrajectoryIsSha256(foundHash) Or Mid$(CStr(line), startAt + 64, 1) <> """" Then Err.Raise vbObjectError + 8819, "WF_TrajectoryRequestHashFromDiagnostics", "INVALID RESULT: diagnostics contain an invalid request_hash"
            If Len(WF_TrajectoryRequestHashFromDiagnostics) = 0 Then WF_TrajectoryRequestHashFromDiagnostics = foundHash
            If StrComp(WF_TrajectoryRequestHashFromDiagnostics, foundHash, vbBinaryCompare) <> 0 Then Err.Raise vbObjectError + 8820, "WF_TrajectoryRequestHashFromDiagnostics", "INVALID RESULT: diagnostics request_hash values disagree"
        End If
    Next line
    If Len(WF_TrajectoryRequestHashFromDiagnostics) = 0 Then Err.Raise vbObjectError + 8821, "WF_TrajectoryRequestHashFromDiagnostics", "INVALID RESULT: diagnostics are empty"
End Function

Private Sub WF_ParseAndValidateTrajectoryBridge(ByVal bridgeText As String, ByVal expectedRequestHash As String, ByVal expectedAnalysisId As String, ByVal expectedIds As Object, ByRef staged As WF_TrajectoryBridgeStage)
    Dim lines As Variant, fields As Variant, lineIndex As Long, recordKind As String, currentPhase As Long
    Dim planExpected As Object, surveyExpected As Object, targetExpected As Object, slideExpected As Object, formationExpected As Object
    Dim planSeen As Object, surveySeen As Object, residualSeen As Object, targetSeen As Object, slideSeen As Object, formationSeen As Object
    Dim planRecords() As Variant, surveyRecords() As Variant, residualRecords() As Variant
    Dim targetRecords() As Variant, slideRecords() As Variant, formationRecords() As Variant, projectionRecord() As Variant
    Dim headerCount As Long, projectionCount As Long, expectedProjectionCount As Long

    If Len(bridgeText) = 0 Or Right$(bridgeText, 1) <> vbLf Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: bridge must end with one newline"
    If InStr(1, bridgeText, vbCr, vbBinaryCompare) > 0 Or InStr(1, bridgeText, Chr$(0), vbBinaryCompare) > 0 Then WF_TrajectoryBridgeError "INVALID RESULT: bridge contains a forbidden control character"
    If Not WF_TrajectoryIsSha256(expectedRequestHash) Then WF_TrajectoryBridgeError "INVALID BRIDGE REQUEST HASH"

    Set planExpected = expectedIds.Item("P")
    Set surveyExpected = expectedIds.Item("S")
    Set targetExpected = expectedIds.Item("T")
    Set slideExpected = expectedIds.Item("L")
    Set formationExpected = expectedIds.Item("F")
    expectedProjectionCount = CLng(expectedIds.Item("X"))
    Set planSeen = WF_TrajectoryNewSet()
    Set surveySeen = WF_TrajectoryNewSet()
    Set residualSeen = WF_TrajectoryNewSet()
    Set targetSeen = WF_TrajectoryNewSet()
    Set slideSeen = WF_TrajectoryNewSet()
    Set formationSeen = WF_TrajectoryNewSet()

    ReDim planRecords(1 To WF_TrajectoryArraySize(planExpected.Count), 1 To 12)
    ReDim surveyRecords(1 To WF_TrajectoryArraySize(surveyExpected.Count), 1 To 12)
    ReDim residualRecords(1 To WF_TrajectoryArraySize(surveyExpected.Count), 1 To 10)
    ReDim targetRecords(1 To WF_TrajectoryArraySize(targetExpected.Count), 1 To 12)
    ReDim slideRecords(1 To WF_TrajectoryArraySize(slideExpected.Count), 1 To 11)
    ReDim formationRecords(1 To WF_TrajectoryArraySize(formationExpected.Count), 1 To 6)
    ReDim projectionRecord(1 To 1, 1 To 13)

    lines = Split(bridgeText, vbLf)
    If UBound(lines) < 1 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: bridge is empty"
    For lineIndex = 0 To UBound(lines) - 1
        If Len(CStr(lines(lineIndex))) = 0 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: blank record"
        fields = Split(CStr(lines(lineIndex)), vbTab)
        recordKind = CStr(fields(0))
        If lineIndex = 0 And recordKind <> "H" Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: header must be first"
        Select Case recordKind
            Case "H"
                If lineIndex <> 0 Or headerCount <> 0 Then WF_TrajectoryBridgeError "DUPLICATE BRIDGE ID: duplicate header"
                WF_ParseTrajectoryHeader fields, expectedRequestHash, expectedAnalysisId, staged
                headerCount = headerCount + 1
            Case "P"
                WF_TrajectoryRequirePhase currentPhase, 1
                WF_ParseTrajectoryStation fields, planExpected, planSeen, planRecords, "plan"
            Case "S"
                WF_TrajectoryRequirePhase currentPhase, 2
                WF_ParseTrajectoryStation fields, surveyExpected, surveySeen, surveyRecords, "survey"
            Case "R"
                WF_TrajectoryRequirePhase currentPhase, 3
                WF_ParseTrajectoryResidual fields, surveyExpected, residualSeen, surveyRecords, residualRecords
            Case "T"
                WF_TrajectoryRequirePhase currentPhase, 4
                WF_ParseTrajectoryTarget fields, targetExpected, targetSeen, targetRecords
            Case "L"
                WF_TrajectoryRequirePhase currentPhase, 5
                WF_ParseTrajectorySlide fields, slideExpected, slideSeen, slideRecords
            Case "F"
                WF_TrajectoryRequirePhase currentPhase, 6
                WF_ParseTrajectoryFormation fields, formationExpected, formationSeen, formationRecords
            Case "X"
                WF_TrajectoryRequirePhase currentPhase, 7
                If projectionCount <> 0 Then WF_TrajectoryBridgeError "DUPLICATE BRIDGE ID: duplicate projection"
                WF_ParseTrajectoryProjection fields, projectionRecord
                projectionCount = projectionCount + 1
            Case Else
                WF_TrajectoryBridgeError "UNKNOWN BRIDGE RECORD: " & recordKind
        End Select
    Next lineIndex

    If headerCount <> 1 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: header"
    WF_TrajectoryRequireCompleteIds planExpected, planSeen, "plan"
    WF_TrajectoryRequireCompleteIds surveyExpected, surveySeen, "survey"
    WF_TrajectoryRequireCompleteIds surveyExpected, residualSeen, "residual"
    WF_TrajectoryRequireCompleteIds targetExpected, targetSeen, "target"
    WF_TrajectoryRequireCompleteIds slideExpected, slideSeen, "slide"
    WF_TrajectoryRequireCompleteIds formationExpected, formationSeen, "formation"
    If projectionCount <> expectedProjectionCount Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: projection count does not match request"
    WF_TrajectoryCheckBridgeCapacity planSeen.Count, surveySeen.Count, targetSeen.Count, slideSeen.Count, formationSeen.Count

    staged.PlanCount = planSeen.Count
    staged.SurveyCount = surveySeen.Count
    staged.TargetCount = targetSeen.Count
    staged.SlideCount = slideSeen.Count
    staged.FormationCount = formationSeen.Count
    staged.ProjectionCount = projectionCount
    staged.PlanRecords = planRecords
    staged.SurveyRecords = surveyRecords
    staged.ResidualRecords = residualRecords
    staged.TargetRecords = targetRecords
    staged.SlideRecords = slideRecords
    staged.FormationRecords = formationRecords
    staged.ProjectionRecord = projectionRecord
End Sub

Private Sub WF_ParseTrajectoryHeader(ByVal fields As Variant, ByVal expectedRequestHash As String, ByVal expectedAnalysisId As String, ByRef staged As WF_TrajectoryBridgeStage)
    If UBound(fields) <> 7 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: invalid header field count"
    If CStr(fields(1)) <> "1.0.0" Then WF_TrajectoryBridgeError "INVALID BRIDGE VERSION"
    If StrComp(CStr(fields(2)), expectedAnalysisId, vbTextCompare) <> 0 Or Not WF_TrajectoryIsUuid(CStr(fields(2))) Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: analysis identity mismatch"
    If StrComp(CStr(fields(3)), expectedRequestHash, vbBinaryCompare) <> 0 Then WF_TrajectoryBridgeError "INVALID BRIDGE REQUEST HASH"
    If Not WF_TrajectoryIsSha256(CStr(fields(4))) Then WF_TrajectoryBridgeError "INVALID BRIDGE RESULT HASH"
    If Len(Trim$(CStr(fields(5)))) = 0 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: engine version"
    WF_TrajectoryRequireEnum CStr(fields(6)), "result status", "complete", "complete_with_warnings"
    If CStr(fields(7)) <> "true" Then WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: deterministic"
    staged.AnalysisId = LCase$(CStr(fields(2)))
    staged.RequestHash = CStr(fields(3))
    staged.ResultHash = CStr(fields(4))
    staged.EngineVersion = CStr(fields(5))
    staged.ResultStatus = CStr(fields(6))
End Sub

Private Sub WF_ParseTrajectoryStation(ByVal fields As Variant, ByVal expected As Object, ByVal seen As Object, ByRef records() As Variant, ByVal stationGroup As String)
    Dim identity As String, stationKind As String, slot As Long, fieldIndex As Long, numberValue As Double
    If UBound(fields) <> 12 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: invalid " & stationGroup & " station field count"
    identity = WF_TrajectoryBridgeIdentity(fields(1), stationGroup)
    slot = WF_TrajectoryRegisterBridgeId(identity, expected, seen, stationGroup)
    If slot <> seen.Count Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: " & stationGroup & " station order"
    stationKind = CStr(fields(2))
    If stationGroup = "plan" Then
        WF_TrajectoryRequireEnum stationKind, "plan station kind", "tie_in", "plan"
        If (slot = 1 And stationKind <> "tie_in") Or (slot > 1 And stationKind <> "plan") Then WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: plan station order"
    Else
        WF_TrajectoryRequireEnum stationKind, "survey station kind", "tie_in", "survey"
        If (slot = 1 And stationKind <> "tie_in") Or (slot > 1 And stationKind <> "survey") Then WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: survey station order"
    End If
    records(slot, 1) = identity
    records(slot, 2) = stationKind
    For fieldIndex = 3 To 12
        numberValue = WF_TrajectoryBridgeNumber(fields(fieldIndex))
        records(slot, fieldIndex) = numberValue
    Next fieldIndex
    If CDbl(records(slot, 3)) < 0# Then WF_TrajectoryBridgeError "NON-FINITE BRIDGE NUMBER: negative station MD"
    If CDbl(records(slot, 4)) < 0# Or CDbl(records(slot, 4)) > WF_TRAJECTORY_PI Then WF_TrajectoryBridgeError "NON-FINITE BRIDGE NUMBER: station inclination range"
    If CDbl(records(slot, 5)) < 0# Or CDbl(records(slot, 5)) >= WF_TRAJECTORY_TWO_PI Then WF_TrajectoryBridgeError "NON-FINITE BRIDGE NUMBER: station azimuth range"
    If CDbl(records(slot, 9)) < 0# Or CDbl(records(slot, 10)) < 0# Or CDbl(records(slot, 11)) <= 0# Or CDbl(records(slot, 12)) < 0# Then WF_TrajectoryBridgeError "NON-FINITE BRIDGE NUMBER: station interval range"
End Sub

Private Sub WF_ParseTrajectoryResidual(ByVal fields As Variant, ByVal expected As Object, ByVal seen As Object, ByRef surveyRecords() As Variant, ByRef records() As Variant)
    Dim identity As String, status As String, slot As Long, fieldIndex As Long
    If UBound(fields) <> 10 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: invalid residual field count"
    identity = WF_TrajectoryBridgeIdentity(fields(1), "residual")
    slot = WF_TrajectoryRegisterBridgeId(identity, expected, seen, "residual")
    If slot <> seen.Count Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: residual order"
    records(slot, 1) = identity
    records(slot, 2) = WF_TrajectoryBridgeNumber(fields(2))
    If Abs(CDbl(records(slot, 2)) - CDbl(surveyRecords(slot, 3))) > 0.000000001 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: residual MD mismatch"
    status = CStr(fields(3))
    WF_TrajectoryRequireInterpolationEnum status
    records(slot, 3) = status
    For fieldIndex = 4 To 10
        records(slot, fieldIndex) = WF_TrajectoryOptionalBridgeNumber(fields(fieldIndex))
    Next fieldIndex
    If status = "ok" Then
        WF_TrajectoryRequirePresentRange records, slot, 4, 10, "residual"
    Else
        WF_TrajectoryRequireEmptyRange records, slot, 4, 10, "residual"
    End If
End Sub

Private Sub WF_ParseTrajectoryTarget(ByVal fields As Variant, ByVal expected As Object, ByVal seen As Object, ByRef records() As Variant)
    Dim identity As String, basis As String, evaluationStatus As String, slot As Long, fieldIndex As Long
    If UBound(fields) <> 12 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: invalid target field count"
    identity = WF_TrajectoryBridgeIdentity(fields(1), "target")
    slot = WF_TrajectoryRegisterBridgeId(identity, expected, seen, "target")
    If slot <> seen.Count Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: target order"
    records(slot, 1) = identity
    records(slot, 2) = WF_TrajectoryBridgeNumber(fields(2))
    basis = CStr(fields(3))
    WF_TrajectoryRequireEnum basis, "target basis", "actual", "projected", "not_reached"
    records(slot, 3) = basis
    evaluationStatus = CStr(fields(4))
    If Len(evaluationStatus) > 0 Then WF_TrajectoryRequireEnum evaluationStatus, "target status", "hit", "miss", "invalid_geometry", "numerical_overflow"
    records(slot, 4) = evaluationStatus
    For fieldIndex = 5 To 12
        records(slot, fieldIndex) = WF_TrajectoryOptionalBridgeNumber(fields(fieldIndex))
    Next fieldIndex
    If basis = "not_reached" Then
        If Len(evaluationStatus) <> 0 Then WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: target not_reached evaluation"
        WF_TrajectoryRequireEmptyRange records, slot, 5, 12, "target"
    Else
        If Len(evaluationStatus) = 0 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: target evaluation"
        WF_TrajectoryRequirePresentRange records, slot, 5, 7, "target position"
        If evaluationStatus = "hit" Or evaluationStatus = "miss" Then
            WF_TrajectoryRequirePresentRange records, slot, 8, 8, "target utilization"
            WF_TrajectoryRequirePresentRange records, slot, 10, 12, "target geometry"
        Else
            WF_TrajectoryRequireEmptyRange records, slot, 8, 12, "invalid target evaluation"
        End If
    End If
End Sub

Private Sub WF_ParseTrajectorySlide(ByVal fields As Variant, ByVal expected As Object, ByVal seen As Object, ByRef records() As Variant)
    Dim identity As String, startStatus As String, endStatus As String, responseStatus As String
    Dim slot As Long, fieldIndex As Long
    If UBound(fields) <> 11 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: invalid slide field count"
    identity = WF_TrajectoryBridgeIdentity(fields(1), "slide")
    slot = WF_TrajectoryRegisterBridgeId(identity, expected, seen, "slide")
    If slot <> seen.Count Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: slide order"
    records(slot, 1) = identity
    startStatus = CStr(fields(2)): endStatus = CStr(fields(3)): responseStatus = CStr(fields(4))
    WF_TrajectoryRequireInterpolationEnum startStatus
    WF_TrajectoryRequireInterpolationEnum endStatus
    If Len(responseStatus) > 0 Then WF_TrajectoryRequireEnum responseStatus, "slide status", "ok", "invalid_slide_length", "low_inclination", "invalid_geometry", "numerical_overflow"
    records(slot, 2) = startStatus: records(slot, 3) = endStatus: records(slot, 4) = responseStatus
    For fieldIndex = 5 To 11
        records(slot, fieldIndex) = WF_TrajectoryOptionalBridgeNumber(fields(fieldIndex))
    Next fieldIndex
    If startStatus = "ok" And endStatus = "ok" Then
        If Len(responseStatus) = 0 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: slide response"
    ElseIf Len(responseStatus) <> 0 Then
        WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: uncovered slide response"
    End If
    If responseStatus = "ok" Then
        WF_TrajectoryRequirePresentRange records, slot, 5, 11, "slide response"
    Else
        WF_TrajectoryRequireEmptyRange records, slot, 5, 11, "slide response"
    End If
End Sub

Private Sub WF_ParseTrajectoryFormation(ByVal fields As Variant, ByVal expected As Object, ByVal seen As Object, ByRef records() As Variant)
    Dim identity As String, coverage As String, sense As String, withinTolerance As String
    Dim slot As Long
    If UBound(fields) <> 6 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: invalid formation field count"
    identity = WF_TrajectoryBridgeIdentity(fields(1), "formation")
    slot = WF_TrajectoryRegisterBridgeId(identity, expected, seen, "formation")
    If slot <> seen.Count Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: formation order"
    records(slot, 1) = identity
    coverage = CStr(fields(2))
    WF_TrajectoryRequireEnum coverage, "formation coverage", "ok", "no_actual_pick", "before_start", "beyond_td", "no_stations", "invalid_measured_depth", "invalid_course", "invalid_geometry", "numerical_overflow"
    records(slot, 2) = coverage
    records(slot, 3) = WF_TrajectoryOptionalBridgeNumber(fields(3))
    records(slot, 4) = WF_TrajectoryOptionalBridgeNumber(fields(4))
    sense = CStr(fields(5))
    If Len(sense) > 0 Then WF_TrajectoryRequireEnum sense, "formation sense", "high", "low", "on_prognosis"
    records(slot, 5) = sense
    withinTolerance = CStr(fields(6))
    If Len(withinTolerance) > 0 And withinTolerance <> "true" And withinTolerance <> "false" Then WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: formation within_tolerance"
    records(slot, 6) = withinTolerance
    If coverage = "ok" Then
        WF_TrajectoryRequirePresentRange records, slot, 3, 4, "formation evaluation"
        If Len(sense) = 0 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: formation sense"
    Else
        WF_TrajectoryRequireEmptyRange records, slot, 3, 4, "formation evaluation"
        If Len(sense) <> 0 Or Len(withinTolerance) <> 0 Then WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: uncovered formation evaluation"
    End If
End Sub

Private Sub WF_ParseTrajectoryProjection(ByVal fields As Variant, ByRef record() As Variant)
    Dim fieldIndex As Long
    If UBound(fields) <> 13 Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: invalid projection field count"
    For fieldIndex = 1 To 12
        record(1, fieldIndex) = WF_TrajectoryBridgeNumber(fields(fieldIndex))
    Next fieldIndex
    If CStr(fields(13)) <> "true" And CStr(fields(13)) <> "false" Then WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: projection low inclination guard"
    record(1, 13) = CStr(fields(13))
End Sub

Private Sub WF_TrajectoryRequireInterpolationEnum(ByVal value As String)
    WF_TrajectoryRequireEnum value, "interpolation status", "ok", "no_stations", "before_start", "beyond_td", "invalid_measured_depth", "invalid_course", "numerical_overflow"
End Sub

Private Sub WF_TrajectoryRequireEnum(ByVal value As String, ByVal fieldName As String, ParamArray allowedValues() As Variant)
    Dim allowed As Variant
    For Each allowed In allowedValues
        If StrComp(value, CStr(allowed), vbBinaryCompare) = 0 Then Exit Sub
    Next allowed
    WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: " & fieldName & " = " & value
End Sub

Private Function WF_TrajectoryRegisterBridgeId(ByVal identity As String, ByVal expected As Object, ByVal seen As Object, ByVal groupName As String) As Long
    If Not expected.Exists(identity) Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: unexpected " & groupName & " id " & identity
    If seen.Exists(identity) Then WF_TrajectoryBridgeError "DUPLICATE BRIDGE ID: " & groupName & " " & identity
    seen.Add identity, True
    WF_TrajectoryRegisterBridgeId = CLng(expected.Item(identity))
End Function

Private Sub WF_TrajectoryRequireCompleteIds(ByVal expected As Object, ByVal seen As Object, ByVal groupName As String)
    Dim identity As Variant
    If expected.Count <> seen.Count Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: " & groupName & " count"
    For Each identity In expected.Keys
        If Not seen.Exists(CStr(identity)) Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: " & groupName & " " & CStr(identity)
    Next identity
End Sub

Private Sub WF_TrajectoryCheckBridgeCapacity(ByVal planCount As Long, ByVal surveyCount As Long, ByVal targetCount As Long, ByVal slideCount As Long, ByVal formationCount As Long)
    If planCount > WF_TRAJECTORY_MAX_PLAN Then WF_TrajectoryBridgeError "BRIDGE CAPACITY EXCEEDED: plan"
    If surveyCount > WF_TRAJECTORY_MAX_SURVEY Then WF_TrajectoryBridgeError "BRIDGE CAPACITY EXCEEDED: survey"
    If targetCount > WF_TRAJECTORY_MAX_TARGETS Then WF_TrajectoryBridgeError "BRIDGE CAPACITY EXCEEDED: targets"
    If slideCount > WF_TRAJECTORY_MAX_SLIDES Then WF_TrajectoryBridgeError "BRIDGE CAPACITY EXCEEDED: slides"
    If formationCount > WF_TRAJECTORY_MAX_FORMATIONS Then WF_TrajectoryBridgeError "BRIDGE CAPACITY EXCEEDED: formations"
End Sub

Private Sub WF_TrajectoryRequirePhase(ByRef currentPhase As Long, ByVal requiredPhase As Long)
    If requiredPhase < currentPhase Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: record kinds out of order"
    currentPhase = requiredPhase
End Sub

Private Function WF_TrajectoryBridgeIdentity(ByVal value As Variant, ByVal groupName As String) As String
    WF_TrajectoryBridgeIdentity = LCase$(CStr(value))
    If Not WF_TrajectoryIsUuid(WF_TrajectoryBridgeIdentity) Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: invalid " & groupName & " UUID"
End Function

Private Function WF_TrajectoryBridgeNumber(ByVal value As Variant) As Double
    Dim textValue As String, expression As Object, parsed As Double
    textValue = CStr(value)
    Set expression = CreateObject("VBScript.RegExp")
    expression.Pattern = "^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?$"
    expression.Global = False
    If Not expression.Test(textValue) Then WF_TrajectoryBridgeError "NON-FINITE BRIDGE NUMBER: " & textValue
    On Error GoTo NonFinite
    parsed = Val(textValue)
    If parsed <> parsed Then GoTo NonFinite
    WF_TrajectoryBridgeNumber = parsed
    Exit Function
NonFinite:
    On Error GoTo 0
    WF_TrajectoryBridgeError "NON-FINITE BRIDGE NUMBER: " & textValue
End Function

Private Function WF_TrajectoryOptionalBridgeNumber(ByVal value As Variant) As Variant
    If Len(CStr(value)) = 0 Then
        WF_TrajectoryOptionalBridgeNumber = Empty
    Else
        WF_TrajectoryOptionalBridgeNumber = WF_TrajectoryBridgeNumber(value)
    End If
End Function

Private Sub WF_TrajectoryRequirePresentRange(ByRef records() As Variant, ByVal rowIndex As Long, ByVal firstColumn As Long, ByVal lastColumn As Long, ByVal groupName As String)
    Dim columnIndex As Long
    For columnIndex = firstColumn To lastColumn
        If IsEmpty(records(rowIndex, columnIndex)) Then WF_TrajectoryBridgeError "MISSING BRIDGE RECORD: " & groupName & " numeric field"
    Next columnIndex
End Sub

Private Sub WF_TrajectoryRequireEmptyRange(ByRef records() As Variant, ByVal rowIndex As Long, ByVal firstColumn As Long, ByVal lastColumn As Long, ByVal groupName As String)
    Dim columnIndex As Long
    For columnIndex = firstColumn To lastColumn
        If Not IsEmpty(records(rowIndex, columnIndex)) Then WF_TrajectoryBridgeError "INVALID BRIDGE ENUM: unexpected " & groupName & " numeric field"
    Next columnIndex
End Sub

Private Sub WF_CommitTrajectoryBridge(ByRef staged As WF_TrajectoryBridgeStage, ByVal requestPath As String, ByVal resultPath As String, ByVal diagnosticPath As String, ByVal executableHash As String, ByVal acceptedUtc As String)
    Dim snapshots As Collection, plan As Variant, survey As Variant, residuals As Variant
    Dim targets As Variant, slides As Variant, formations As Variant, projection As Variant
    Dim planCalc() As Variant, planVisible() As Variant, surveyCalc() As Variant, surveyVisible() As Variant
    Dim compareValues() As Variant, contractValues() As Variant, graphMain() As Variant, graphResidual() As Variant, graphError() As Variant
    Dim targetValues() As Variant, targetHelper() As Variant, slideValues() As Variant, formationValues() As Variant
    Dim slideGraph() As Variant, targetGraph() As Variant, projectionValues(1 To 40, 1 To 1) As Variant
    Dim checks(1 To 20, 1 To 3) As Variant, resultMetrics(1 To 10, 1 To 1) As Variant, terminalMetrics(1 To 3, 1 To 1) As Variant
    Dim summaryMetrics(1 To 5, 1 To 1) As Variant, summaryUnits(1 To 3, 1 To 1) As Variant, evidence(1 To 10, 1 To 1) As Variant
    Dim rowIndex As Long, previousNorth As Double, previousEast As Double, previousTvd As Double
    Dim northValue As Double, eastValue As Double, tvdValue As Double, verticalSection As Double, crossline As Double
    Dim planNorth As Double, planEast As Double, planTvd As Double
    Dim surfaceNorth As Double, surfaceEast As Double, verticalSectionAzimuth As Double
    Dim lengthFactor As Double, gradientFactor As Double, angleFactor As Double, dlsLimit As Double
    Dim minimumSlideLength As Double, yieldOutlierLimit As Double
    Dim latestCrossline As Double, latestError3d As Double, terminalHorizontal As Double, maxDls As Double, maxGap As Double
    Dim latestCoverage As String, nextTarget As String, targetStatus As String, slideStatus As String, formationStatus As String
    Dim targetIssues As Long, slideIssues As Long, formationIssues As Long, actualFormationPicks As Long, cautionCount As Long, stopCount As Long
    Dim overallState As String, maxRows As Long, failureNumber As Long, failureDescription As String

    Set snapshots = WF_TrajectoryCaptureSnapshots()

    WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED = False
    On Error GoTo Rollback
    plan = staged.PlanRecords
    survey = staged.SurveyRecords
    residuals = staged.ResidualRecords
    targets = staged.TargetRecords
    slides = staged.SlideRecords
    formations = staged.FormationRecords
    projection = staged.ProjectionRecord
    lengthFactor = WF_UnitFactor("Length")
    gradientFactor = WF_UnitFactor("Angular gradient")
    angleFactor = WF_UnitFactor("Angle")
    surfaceNorth = WF_ToSI(WF_TrajectoryInputNumber("B13", "surface north"), WF_Str("Inputs", "E5", "m"))
    surfaceEast = WF_ToSI(WF_TrajectoryInputNumber("B14", "surface east"), WF_Str("Inputs", "E5", "m"))
    verticalSectionAzimuth = WF_Mod2Pi(WF_ToSI(WF_TrajectoryInputNumber("B16", "vertical section azimuth"), WF_Str("Inputs", "E6", "rad")))
    dlsLimit = WF_ToSI(WF_TrajectoryInputNumber("H5", "DLS operating limit"), WF_Str("Inputs", "H6", "rad/m"))
    minimumSlideLength = WF_ToSI(WF_TrajectoryInputNumber("N6", "minimum slide length"), WF_Str("Inputs", "E10", "m"))
    yieldOutlierLimit = WF_ToSI(WF_TrajectoryInputNumber("N7", "slide-yield outlier limit"), WF_Str("Inputs", "K9", "rad/m"))
    maxRows = CLng(Application.Max(staged.PlanCount, staged.SurveyCount))

    ReDim planCalc(1 To staged.PlanCount, 1 To 18)
    ReDim planVisible(1 To staged.PlanCount, 1 To 8)
    ReDim surveyCalc(1 To staged.SurveyCount, 1 To 18)
    ReDim surveyVisible(1 To staged.SurveyCount, 1 To 20)
    ReDim compareValues(1 To staged.SurveyCount, 1 To 34)
    ReDim contractValues(1 To staged.SurveyCount, 1 To 13)
    ReDim graphMain(1 To maxRows, 1 To 15)
    ReDim graphResidual(1 To staged.SurveyCount, 1 To 5)
    ReDim graphError(1 To staged.SurveyCount, 1 To 3)

    previousNorth = surfaceNorth: previousEast = surfaceEast: previousTvd = 0#
    For rowIndex = 1 To staged.PlanCount
        northValue = CDbl(plan(rowIndex, 6)) + surfaceNorth
        eastValue = CDbl(plan(rowIndex, 7)) + surfaceEast
        tvdValue = CDbl(plan(rowIndex, 8))
        verticalSection = northValue * Cos(verticalSectionAzimuth) + eastValue * Sin(verticalSectionAzimuth)
        crossline = -northValue * Sin(verticalSectionAzimuth) + eastValue * Cos(verticalSectionAzimuth)
        WF_TrajectoryFillStationCalc planCalc, rowIndex, ThisWorkbook.Worksheets("Plan").Cells(rowIndex + 6, 1).Value2, plan, northValue, eastValue, tvdValue, previousNorth, previousEast, previousTvd
        planVisible(rowIndex, 1) = True
        planVisible(rowIndex, 2) = tvdValue * lengthFactor
        planVisible(rowIndex, 3) = northValue * lengthFactor
        planVisible(rowIndex, 4) = eastValue * lengthFactor
        planVisible(rowIndex, 5) = verticalSection * lengthFactor
        planVisible(rowIndex, 6) = crossline * lengthFactor
        planVisible(rowIndex, 7) = CDbl(plan(rowIndex, 12)) * gradientFactor
        planVisible(rowIndex, 8) = "OK"
        graphMain(rowIndex, 1) = eastValue * lengthFactor
        graphMain(rowIndex, 2) = northValue * lengthFactor
        graphMain(rowIndex, 5) = verticalSection * lengthFactor
        graphMain(rowIndex, 6) = tvdValue * lengthFactor
        graphMain(rowIndex, 14) = CDbl(plan(rowIndex, 12)) * gradientFactor
        previousNorth = northValue: previousEast = eastValue: previousTvd = tvdValue
    Next rowIndex

    previousNorth = surfaceNorth: previousEast = surfaceEast: previousTvd = 0#
    For rowIndex = 1 To staged.SurveyCount
        northValue = CDbl(survey(rowIndex, 6)) + surfaceNorth
        eastValue = CDbl(survey(rowIndex, 7)) + surfaceEast
        tvdValue = CDbl(survey(rowIndex, 8))
        verticalSection = northValue * Cos(verticalSectionAzimuth) + eastValue * Sin(verticalSectionAzimuth)
        crossline = -northValue * Sin(verticalSectionAzimuth) + eastValue * Cos(verticalSectionAzimuth)
        WF_TrajectoryFillStationCalc surveyCalc, rowIndex, ThisWorkbook.Worksheets("Survey").Cells(rowIndex + 6, 1).Value2, survey, northValue, eastValue, tvdValue, previousNorth, previousEast, previousTvd
        surveyVisible(rowIndex, 1) = True
        surveyVisible(rowIndex, 2) = tvdValue * lengthFactor
        surveyVisible(rowIndex, 3) = northValue * lengthFactor
        surveyVisible(rowIndex, 4) = eastValue * lengthFactor
        surveyVisible(rowIndex, 5) = verticalSection * lengthFactor
        surveyVisible(rowIndex, 6) = crossline * lengthFactor
        latestCoverage = WF_TrajectoryDisplayEnum(CStr(residuals(rowIndex, 3)))
        compareValues(rowIndex, 1) = latestCoverage
        compareValues(rowIndex, 25) = "PRESENTATION_ONLY_NOT_RUST_RESULT"
        compareValues(rowIndex, 26) = "PRESENTATION_ONLY_NOT_RUST_RESULT"
        If CStr(residuals(rowIndex, 3)) = "ok" Then
            planTvd = tvdValue - CDbl(residuals(rowIndex, 6))
            planNorth = northValue - CDbl(residuals(rowIndex, 4))
            planEast = eastValue - CDbl(residuals(rowIndex, 5))
            surveyVisible(rowIndex, 7) = planTvd * lengthFactor
            surveyVisible(rowIndex, 8) = planNorth * lengthFactor
            surveyVisible(rowIndex, 9) = planEast * lengthFactor
            surveyVisible(rowIndex, 10) = CDbl(residuals(rowIndex, 6)) * lengthFactor
            surveyVisible(rowIndex, 11) = CDbl(residuals(rowIndex, 4)) * lengthFactor
            surveyVisible(rowIndex, 12) = CDbl(residuals(rowIndex, 5)) * lengthFactor
            surveyVisible(rowIndex, 13) = CDbl(residuals(rowIndex, 7)) * lengthFactor
            surveyVisible(rowIndex, 14) = CDbl(residuals(rowIndex, 7)) * lengthFactor
            surveyVisible(rowIndex, 15) = CDbl(residuals(rowIndex, 8)) * lengthFactor
            surveyVisible(rowIndex, 16) = CDbl(residuals(rowIndex, 9)) * lengthFactor
            surveyVisible(rowIndex, 17) = CDbl(residuals(rowIndex, 10)) * lengthFactor
            compareValues(rowIndex, 22) = planTvd
            compareValues(rowIndex, 23) = planNorth
            compareValues(rowIndex, 24) = planEast
            compareValues(rowIndex, 27) = CDbl(residuals(rowIndex, 6))
            compareValues(rowIndex, 28) = CDbl(residuals(rowIndex, 4))
            compareValues(rowIndex, 29) = CDbl(residuals(rowIndex, 5))
            compareValues(rowIndex, 30) = CDbl(residuals(rowIndex, 7))
            compareValues(rowIndex, 31) = CDbl(residuals(rowIndex, 7))
            compareValues(rowIndex, 32) = CDbl(residuals(rowIndex, 8))
            compareValues(rowIndex, 33) = CDbl(residuals(rowIndex, 9))
            compareValues(rowIndex, 34) = CDbl(residuals(rowIndex, 10))
            latestCrossline = CDbl(residuals(rowIndex, 8))
            latestError3d = CDbl(residuals(rowIndex, 10))
            terminalHorizontal = CDbl(residuals(rowIndex, 9))
        End If
        surveyVisible(rowIndex, 18) = latestCoverage
        surveyVisible(rowIndex, 19) = CDbl(survey(rowIndex, 12)) * gradientFactor
        surveyVisible(rowIndex, 20) = "OK"
        If CDbl(survey(rowIndex, 12)) > maxDls Then maxDls = CDbl(survey(rowIndex, 12))
        If rowIndex > 1 And CDbl(survey(rowIndex, 3)) - CDbl(survey(rowIndex - 1, 3)) > maxGap Then maxGap = CDbl(survey(rowIndex, 3)) - CDbl(survey(rowIndex - 1, 3))
        WF_TrajectoryFillContractRow contractValues, rowIndex, survey, northValue, eastValue, tvdValue
        graphMain(rowIndex, 3) = eastValue * lengthFactor
        graphMain(rowIndex, 4) = northValue * lengthFactor
        graphMain(rowIndex, 7) = verticalSection * lengthFactor
        graphMain(rowIndex, 8) = tvdValue * lengthFactor
        graphMain(rowIndex, 9) = CDbl(survey(rowIndex, 3)) * lengthFactor
        graphMain(rowIndex, 10) = CDbl(survey(rowIndex, 4)) * angleFactor
        graphMain(rowIndex, 11) = CDbl(survey(rowIndex, 5)) * angleFactor
        graphMain(rowIndex, 13) = CDbl(survey(rowIndex, 3)) * lengthFactor
        graphMain(rowIndex, 15) = CDbl(survey(rowIndex, 12)) * gradientFactor
        graphResidual(rowIndex, 1) = CDbl(survey(rowIndex, 3)) * lengthFactor
        graphError(rowIndex, 1) = CDbl(survey(rowIndex, 3)) * lengthFactor
        If CStr(residuals(rowIndex, 3)) = "ok" Then
            graphResidual(rowIndex, 2) = CDbl(residuals(rowIndex, 6)) * lengthFactor
            graphResidual(rowIndex, 3) = CDbl(residuals(rowIndex, 7)) * lengthFactor
            graphResidual(rowIndex, 4) = CDbl(residuals(rowIndex, 7)) * lengthFactor
            graphResidual(rowIndex, 5) = CDbl(residuals(rowIndex, 8)) * lengthFactor
            graphError(rowIndex, 2) = CDbl(residuals(rowIndex, 9)) * lengthFactor
            graphError(rowIndex, 3) = CDbl(residuals(rowIndex, 10)) * lengthFactor
        End If
        previousNorth = northValue: previousEast = eastValue: previousTvd = tvdValue
    Next rowIndex

    ReDim targetValues(1 To WF_TrajectoryArraySize(staged.TargetCount), 1 To 6)
    ReDim targetHelper(1 To WF_TrajectoryArraySize(staged.TargetCount), 1 To 19)
    ReDim targetGraph(1 To WF_TrajectoryArraySize(staged.TargetCount), 1 To 3)
    nextTarget = "NO TARGET"
    For rowIndex = 1 To staged.TargetCount
        targetStatus = WF_TrajectoryTargetStatus(targets, rowIndex)
        targetValues(rowIndex, 1) = WF_TrajectoryDisplayEnum(CStr(targets(rowIndex, 3)))
        targetValues(rowIndex, 2) = WF_TrajectoryDisplayOptional(targets(rowIndex, 10), lengthFactor)
        targetValues(rowIndex, 3) = WF_TrajectoryDisplayOptional(targets(rowIndex, 11), lengthFactor)
        targetValues(rowIndex, 4) = targets(rowIndex, 8)
        targetValues(rowIndex, 5) = targets(rowIndex, 9)
        targetValues(rowIndex, 6) = targetStatus
        targetHelper(rowIndex, 1) = targets(rowIndex, 1)
        targetHelper(rowIndex, 2) = targets(rowIndex, 2)
        targetHelper(rowIndex, 3) = targets(rowIndex, 3)
        targetHelper(rowIndex, 4) = "UNAVAILABLE_NOT_IN_RUST_BRIDGE"
        targetHelper(rowIndex, 5) = "UNAVAILABLE_NOT_IN_RUST_BRIDGE"
        targetHelper(rowIndex, 6) = "UNAVAILABLE_NOT_IN_RUST_BRIDGE"
        targetHelper(rowIndex, 7) = "UNAVAILABLE_NOT_IN_RUST_BRIDGE"
        targetHelper(rowIndex, 8) = targets(rowIndex, 7)
        targetHelper(rowIndex, 9) = targets(rowIndex, 5)
        targetHelper(rowIndex, 10) = targets(rowIndex, 6)
        targetHelper(rowIndex, 11) = "UNAVAILABLE_NOT_IN_RUST_BRIDGE"
        targetHelper(rowIndex, 12) = "UNAVAILABLE_NOT_IN_RUST_BRIDGE"
        targetHelper(rowIndex, 13) = targets(rowIndex, 12)
        targetHelper(rowIndex, 14) = targets(rowIndex, 10)
        targetHelper(rowIndex, 15) = targets(rowIndex, 11)
        targetHelper(rowIndex, 16) = targets(rowIndex, 8)
        targetHelper(rowIndex, 17) = targets(rowIndex, 9)
        targetHelper(rowIndex, 18) = targets(rowIndex, 4)
        targetHelper(rowIndex, 19) = targetStatus
        targetGraph(rowIndex, 1) = ThisWorkbook.Worksheets("Targets").Cells(rowIndex + 6, 1).Value2
        targetGraph(rowIndex, 2) = targets(rowIndex, 8)
        targetGraph(rowIndex, 3) = targets(rowIndex, 9)
        If rowIndex = 1 Then nextTarget = CStr(targetGraph(rowIndex, 1)) & ": " & targetStatus
        If CStr(targets(rowIndex, 4)) = "invalid_geometry" Or CStr(targets(rowIndex, 4)) = "numerical_overflow" Then targetIssues = targetIssues + 1
    Next rowIndex

    ReDim slideValues(1 To WF_TrajectoryArraySize(staged.SlideCount), 1 To 9)
    ReDim slideGraph(1 To WF_TrajectoryArraySize(staged.SlideCount), 1 To 3)
    slideStatus = "OK"
    For rowIndex = 1 To staged.SlideCount
        slideValues(rowIndex, 1) = WF_TrajectoryDisplayOptional(slides(rowIndex, 5), gradientFactor)
        slideValues(rowIndex, 2) = WF_TrajectoryDisplayOptional(slides(rowIndex, 6), gradientFactor)
        slideValues(rowIndex, 3) = WF_TrajectoryDisplayOptional(slides(rowIndex, 7), gradientFactor)
        slideValues(rowIndex, 4) = WF_TrajectoryDisplayOptional(slides(rowIndex, 8), gradientFactor)
        slideValues(rowIndex, 5) = WF_TrajectoryDisplayOptional(slides(rowIndex, 9), gradientFactor)
        slideValues(rowIndex, 6) = WF_TrajectoryDisplayOptional(slides(rowIndex, 10), angleFactor)
        slideValues(rowIndex, 7) = WF_TrajectoryDisplayOptional(slides(rowIndex, 11), angleFactor)
        slideValues(rowIndex, 8) = Empty
        If CStr(slides(rowIndex, 4)) = "ok" Then
            If WF_ToSI(WF_TrajectoryRequiredNumber(ThisWorkbook.Worksheets("Slide Performance").Cells(rowIndex + 6, 5).Value2, "slide length"), WF_Str("Inputs", "E10", "m")) < minimumSlideLength Then
                slideValues(rowIndex, 9) = "SHORT SLIDE"
            ElseIf Not IsEmpty(slides(rowIndex, 9)) And CDbl(slides(rowIndex, 9)) > yieldOutlierLimit Then
                slideValues(rowIndex, 9) = "OUTLIER"
            Else
                slideValues(rowIndex, 9) = "OK"
            End If
        ElseIf Len(CStr(slides(rowIndex, 4))) > 0 Then
            slideValues(rowIndex, 9) = WF_TrajectoryDisplayEnum(CStr(slides(rowIndex, 4)))
        Else
            slideValues(rowIndex, 9) = "OUTSIDE SURVEY"
        End If
        slideGraph(rowIndex, 1) = ThisWorkbook.Worksheets("Slide Performance").Cells(rowIndex + 6, 1).Value2
        slideGraph(rowIndex, 2) = slideValues(rowIndex, 5)
        slideGraph(rowIndex, 3) = yieldOutlierLimit * gradientFactor
        If CStr(slideValues(rowIndex, 9)) <> "OK" Then slideIssues = slideIssues + 1
    Next rowIndex
    If slideIssues > 0 Then slideStatus = "REVIEW"

    ReDim formationValues(1 To WF_TrajectoryArraySize(staged.FormationCount), 1 To 5)
    formationStatus = "OK"
    For rowIndex = 1 To staged.FormationCount
        formationValues(rowIndex, 1) = WF_TrajectoryDisplayOptional(formations(rowIndex, 3), lengthFactor)
        formationValues(rowIndex, 2) = WF_TrajectoryDisplayOptional(formations(rowIndex, 4), lengthFactor)
        formationValues(rowIndex, 3) = WF_TrajectoryDisplayEnum(CStr(formations(rowIndex, 5)))
        formationValues(rowIndex, 4) = WF_TrajectoryDisplayEnum(CStr(formations(rowIndex, 2)))
        If CStr(formations(rowIndex, 6)) = "true" Then
            formationValues(rowIndex, 5) = "OK"
        ElseIf CStr(formations(rowIndex, 6)) = "false" Then
            formationValues(rowIndex, 5) = "OUTSIDE TOLERANCE"
            formationIssues = formationIssues + 1
        Else
            formationValues(rowIndex, 5) = formationValues(rowIndex, 4)
        End If
        If CStr(formations(rowIndex, 2)) <> "no_actual_pick" Then actualFormationPicks = actualFormationPicks + 1
        If CStr(formations(rowIndex, 2)) <> "ok" And CStr(formations(rowIndex, 2)) <> "no_actual_pick" Then formationIssues = formationIssues + 1
    Next rowIndex
    If formationIssues > 0 Then
        formationStatus = "REVIEW"
    ElseIf staged.FormationCount = 0 Then
        formationStatus = "NO FORMATIONS"
    ElseIf actualFormationPicks = 0 Then
        formationStatus = "NO ACTUAL PICKS"
    End If

    WF_TrajectoryPrepareProjectionValues staged, survey, projection, projectionValues, surfaceNorth, surfaceEast
    WF_TrajectoryPrepareChecks checks, staged, maxGap, latestCoverage, maxDls, dlsLimit, targetIssues, slideIssues, formationIssues, stopCount, cautionCount
    If stopCount > 0 Then
        overallState = "STOP"
    ElseIf cautionCount > 0 Then
        overallState = "CAUTION"
    Else
        overallState = "READY"
    End If

    resultMetrics(1, 1) = CDbl(survey(staged.SurveyCount, 3)) * lengthFactor
    resultMetrics(2, 1) = latestCrossline * lengthFactor
    resultMetrics(3, 1) = latestError3d * lengthFactor
    resultMetrics(4, 1) = maxDls * gradientFactor
    resultMetrics(5, 1) = latestCoverage
    resultMetrics(6, 1) = nextTarget
    resultMetrics(7, 1) = slideStatus
    resultMetrics(8, 1) = formationStatus
    resultMetrics(9, 1) = IIf(staged.ProjectionCount = 1, "RUST TENDENCY PROJECTION", "LATEST VALID SURVEY")
    resultMetrics(10, 1) = "DETERMINISTIC - NO UNCERTAINTY MODEL"
    terminalMetrics(1, 1) = latestCrossline * lengthFactor
    terminalMetrics(2, 1) = terminalHorizontal * lengthFactor
    terminalMetrics(3, 1) = latestError3d * lengthFactor
    summaryMetrics(1, 1) = overallState
    summaryMetrics(2, 1) = resultMetrics(1, 1)
    summaryMetrics(3, 1) = resultMetrics(2, 1)
    summaryMetrics(4, 1) = resultMetrics(3, 1)
    summaryMetrics(5, 1) = Format$(maxDls * gradientFactor, "0.00") & " / " & Format$(dlsLimit * gradientFactor, "0.00") & " " & WF_UnitLabel("Angular gradient")
    summaryUnits(1, 1) = WF_UnitLabel("Length"): summaryUnits(2, 1) = WF_UnitLabel("Length"): summaryUnits(3, 1) = WF_UnitLabel("Length")

    With ThisWorkbook.Worksheets("Plan")
        .Range("F7:M506").ClearContents
        If WF_TRAJECTORY_INJECT_COMMIT_FAILURE Then Err.Raise vbObjectError + 8902, "WF_CommitTrajectoryBridge", "INJECTED COMMIT FAILURE"
        .Range("F7").Resize(staged.PlanCount, 8).Value2 = planVisible
    End With
    With ThisWorkbook.Worksheets("Survey")
        .Range("F7:Y506").ClearContents
        .Range("F7").Resize(staged.SurveyCount, 20).Value2 = surveyVisible
    End With
    With ThisWorkbook.Worksheets("Targets")
        .Range("L7:Q106").ClearContents
        If staged.TargetCount > 0 Then .Range("L7").Resize(staged.TargetCount, 6).Value2 = targetValues
    End With
    With ThisWorkbook.Worksheets("Slide Performance")
        .Range("K7:S206").ClearContents
        If staged.SlideCount > 0 Then .Range("K7").Resize(staged.SlideCount, 9).Value2 = slideValues
    End With
    With ThisWorkbook.Worksheets("Formation Tops")
        .Range("G7:K106").ClearContents
        If staged.FormationCount > 0 Then .Range("G7").Resize(staged.FormationCount, 5).Value2 = formationValues
    End With
    With ThisWorkbook.Worksheets("Results")
        .Range("A26:M525").ClearContents
        .Range("A26").Resize(staged.SurveyCount, 13).Value2 = contractValues
        .Range("B6:B15").Value2 = resultMetrics
        .Range("B19:B21").Value2 = terminalMetrics
    End With
    With ThisWorkbook.Worksheets("Checks")
        .Range("B6:D25").ClearContents
        .Range("B6:D25").Value2 = checks
    End With
    With ThisWorkbook.Worksheets("Summary")
        .Range("B5:B9").Value2 = summaryMetrics
        .Range("C6:C8").Value2 = summaryUnits
        .Range("B10").Value2 = nextTarget & " — " & IIf(overallState = "STOP", "Resolve STOP checks", IIf(overallState = "CAUTION", "Review cautions before proceeding", "Proceed under approved workflow"))
    End With
    With ThisWorkbook.Worksheets("Calc")
        .Range("A7:R506").ClearContents
        .Range("A7").Resize(staged.PlanCount, 18).Value2 = planCalc
        .Range("T7:BS506").ClearContents
        .Range("T7").Resize(staged.SurveyCount, 18).Value2 = surveyCalc
        .Range("AL7").Resize(staged.SurveyCount, 34).Value2 = compareValues
        .Range("FI7:GJ612").ClearContents
        .Range("GL6:GM45").Columns(2).ClearContents
        .Range("GM6:GM45").Value2 = projectionValues
        .Range("HB7:HT106").ClearContents
        If staged.TargetCount > 0 Then .Range("HB7").Resize(staged.TargetCount, 19).Value2 = targetHelper
        .Range("DA7:EG506").ClearContents
        .Range("DA7").Resize(maxRows, 15).Value2 = graphMain
        .Range("DQ7").Resize(staged.SurveyCount, 5).Value2 = graphResidual
        .Range("DW7").Resize(staged.SurveyCount, 3).Value2 = graphError
        If staged.SlideCount > 0 Then .Range("EA7").Resize(staged.SlideCount, 3).Value2 = slideGraph
        If staged.TargetCount > 0 Then .Range("EE7").Resize(staged.TargetCount, 3).Value2 = targetGraph
    End With

    WF_RefreshDirectionalPresentation
    evidence(1, 1) = WF_TRAJECTORY_EXECUTION_MODE
    evidence(2, 1) = IIf(staged.ResultStatus = "complete_with_warnings", "CALCULATED WITH WARNINGS", "CALCULATED")
    evidence(3, 1) = requestPath
    evidence(4, 1) = resultPath
    evidence(5, 1) = diagnosticPath
    evidence(6, 1) = staged.RequestHash
    evidence(7, 1) = staged.ResultHash
    evidence(8, 1) = staged.EngineVersion
    evidence(9, 1) = executableHash
    evidence(10, 1) = acceptedUtc
    ThisWorkbook.Worksheets("Results").Range("P5:P14").Value2 = evidence
    Exit Sub

Rollback:
    failureNumber = Err.Number
    failureDescription = Err.Description
    WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED = WF_TrajectoryRestoreSnapshots(snapshots)
    If WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED Then
        WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED = WF_TrajectorySnapshotsMatch(snapshots)
    End If
    If Not WF_TRAJECTORY_LAST_ROLLBACK_VERIFIED Then
        failureDescription = failureDescription & " | ROLLBACK INCOMPLETE — LAST ACCEPTED VALUES MAY BE PARTIALLY REPLACED"
    End If
    Err.Raise failureNumber, "WF_CommitTrajectoryBridge", failureDescription
End Sub

Private Sub WF_TrajectoryFillStationCalc(ByRef output() As Variant, ByVal rowIndex As Long, ByVal displayId As Variant, ByVal records As Variant, ByVal northValue As Double, ByVal eastValue As Double, ByVal tvdValue As Double, ByVal previousNorth As Double, ByVal previousEast As Double, ByVal previousTvd As Double)
    output(rowIndex, 1) = True
    output(rowIndex, 2) = displayId
    output(rowIndex, 3) = records(rowIndex, 3)
    output(rowIndex, 4) = records(rowIndex, 4)
    output(rowIndex, 5) = records(rowIndex, 5)
    output(rowIndex, 6) = records(rowIndex, 9)
    output(rowIndex, 7) = records(rowIndex, 10)
    output(rowIndex, 8) = records(rowIndex, 11)
    If rowIndex > 1 Then
        output(rowIndex, 9) = tvdValue - previousTvd
        output(rowIndex, 10) = northValue - previousNorth
        output(rowIndex, 11) = eastValue - previousEast
    End If
    output(rowIndex, 12) = tvdValue
    output(rowIndex, 13) = northValue
    output(rowIndex, 14) = eastValue
    output(rowIndex, 15) = "PRESENTATION_ONLY_NOT_RUST_RESULT"
    output(rowIndex, 16) = "PRESENTATION_ONLY_NOT_RUST_RESULT"
    output(rowIndex, 17) = records(rowIndex, 12)
    output(rowIndex, 18) = "OK"
End Sub

Private Sub WF_TrajectoryFillContractRow(ByRef output() As Variant, ByVal rowIndex As Long, ByVal survey As Variant, ByVal northValue As Double, ByVal eastValue As Double, ByVal tvdValue As Double)
    output(rowIndex, 1) = ThisWorkbook.Worksheets("Survey").Cells(rowIndex + 6, 1).Value2
    output(rowIndex, 2) = survey(rowIndex, 3)
    output(rowIndex, 3) = survey(rowIndex, 4)
    output(rowIndex, 4) = survey(rowIndex, 5)
    output(rowIndex, 5) = tvdValue
    output(rowIndex, 6) = northValue
    output(rowIndex, 7) = eastValue
    output(rowIndex, 8) = "PRESENTATION_ONLY_NOT_RUST_RESULT"
    output(rowIndex, 9) = "PRESENTATION_ONLY_NOT_RUST_RESULT"
    output(rowIndex, 10) = survey(rowIndex, 12)
    output(rowIndex, 11) = ThisWorkbook.Worksheets("Survey").Cells(rowIndex + 6, 5).Value2
    output(rowIndex, 12) = "OK"
    output(rowIndex, 13) = ThisWorkbook.Worksheets("Survey").Cells(rowIndex + 6, 26).Value2
End Sub

Private Sub WF_TrajectoryPrepareProjectionValues(ByRef staged As WF_TrajectoryBridgeStage, ByVal survey As Variant, ByVal projection As Variant, ByRef output() As Variant, ByVal surfaceNorth As Double, ByVal surfaceEast As Double)
    output(1, 1) = staged.SurveyCount + 6
    output(2, 1) = survey(staged.SurveyCount, 3)
    output(3, 1) = survey(staged.SurveyCount, 4)
    output(4, 1) = survey(staged.SurveyCount, 5)
    output(5, 1) = survey(staged.SurveyCount, 8)
    output(6, 1) = CDbl(survey(staged.SurveyCount, 6)) + surfaceNorth
    output(7, 1) = CDbl(survey(staged.SurveyCount, 7)) + surfaceEast
    If staged.ProjectionCount = 0 Then
        output(39, 1) = "NOT REQUESTED"
        output(40, 1) = "NOT REQUESTED"
        Exit Sub
    End If
    output(8, 1) = projection(1, 1)
    output(9, 1) = CDbl(projection(1, 7)) - CDbl(projection(1, 1))
    output(10, 1) = WF_ToSI(WF_TrajectoryInputNumber("K7", "projection build tendency"), WF_Str("Inputs", "K9", "rad/m"))
    output(11, 1) = WF_ToSI(WF_TrajectoryInputNumber("K8", "projection effective turn tendency"), WF_Str("Inputs", "K9", "rad/m"))
    output(12, 1) = WF_ToSI(WF_TrajectoryInputNumber("N5", "low inclination threshold"), "deg")
    output(13, 1) = CDbl(projection(1, 1)) - CDbl(survey(staged.SurveyCount, 3))
    output(14, 1) = projection(1, 2)
    output(17, 1) = projection(1, 3)
    output(23, 1) = projection(1, 6)
    output(24, 1) = CDbl(projection(1, 4)) + surfaceNorth
    output(25, 1) = CDbl(projection(1, 5)) + surfaceEast
    output(26, 1) = projection(1, 7)
    output(27, 1) = projection(1, 8)
    output(30, 1) = projection(1, 9)
    output(36, 1) = projection(1, 12)
    output(37, 1) = CDbl(projection(1, 10)) + surfaceNorth
    output(38, 1) = CDbl(projection(1, 11)) + surfaceEast
    output(39, 1) = IIf(CStr(projection(1, 13)) = "true", "LOW - TURN GUARDED", "NORMAL")
    output(40, 1) = "DETERMINISTIC"
End Sub

Private Sub WF_TrajectoryPrepareChecks(ByRef output() As Variant, ByRef staged As WF_TrajectoryBridgeStage, ByVal maxGap As Double, ByVal latestCoverage As String, ByVal maxDls As Double, ByVal dlsLimit As Double, ByVal targetIssues As Long, ByVal slideIssues As Long, ByVal formationIssues As Long, ByRef stopCount As Long, ByRef cautionCount As Long)
    Dim warningGap As Double, rowIndex As Long
    warningGap = WF_ToSI(WF_TrajectoryInputNumber("N9", "survey-gap warning"), WF_Str("Inputs", "E7", "m"))
    WF_TrajectorySetCheck output, 1, "Defined", "PASS", "INFO"
    WF_TrajectorySetCheck output, 2, "Defined", "PASS", "INFO"
    WF_TrajectorySetCheck output, 3, CStr(staged.PlanCount) & " / " & CStr(WF_TRAJECTORY_MAX_PLAN), "PASS", "INFO"
    WF_TrajectorySetCheck output, 4, CStr(staged.SurveyCount) & " / " & CStr(WF_TRAJECTORY_MAX_SURVEY), "PASS", "INFO"
    WF_TrajectorySetCheck output, 5, 0, "PASS", "INFO"
    WF_TrajectorySetCheck output, 6, 0, "PASS", "INFO"
    WF_TrajectorySetCheck output, 7, 0, "PASS", "INFO"
    WF_TrajectorySetCheck output, 8, maxGap * WF_UnitFactor("Length"), IIf(maxGap > warningGap, "CAUTION", "PASS"), IIf(maxGap > warningGap, "CAUTION", "INFO")
    WF_TrajectorySetCheck output, 9, latestCoverage, IIf(latestCoverage = "OK", "PASS", "CAUTION"), IIf(latestCoverage = "OK", "INFO", "CAUTION")
    WF_TrajectorySetCheck output, 10, maxDls * WF_UnitFactor("Angular gradient"), IIf(maxDls <= dlsLimit, "PASS", "CAUTION"), IIf(maxDls <= dlsLimit, "INFO", "CAUTION")
    WF_TrajectorySetCheck output, 11, targetIssues, IIf(targetIssues = 0, "PASS", "STOP"), IIf(targetIssues = 0, "INFO", "STOP")
    WF_TrajectorySetCheck output, 12, slideIssues, IIf(slideIssues = 0, "PASS", "CAUTION"), IIf(slideIssues = 0, "INFO", "CAUTION")
    WF_TrajectorySetCheck output, 13, formationIssues, IIf(formationIssues = 0, "PASS", "CAUTION"), IIf(formationIssues = 0, "INFO", "CAUTION")
    WF_TrajectorySetCheck output, 14, 0, "PASS", "INFO"
    WF_TrajectorySetCheck output, 15, "Hash verified Rust engine", "PASS", "INFO"
    WF_TrajectorySetCheck output, 16, "Not calculated", "INFO", "INFO"
    WF_TrajectorySetCheck output, 17, "Not calculated", "INFO", "INFO"
    WF_TrajectorySetCheck output, 18, "Not calculated", "INFO", "INFO"
    WF_TrajectorySetCheck output, 19, IIf(staged.ProjectionCount = 1, "Deterministic", "Not requested"), "INFO", "INFO"
    WF_TrajectorySetCheck output, 20, staged.ResultHash, IIf(staged.ResultStatus = "complete", "PASS", "CAUTION"), IIf(staged.ResultStatus = "complete", "INFO", "CAUTION")
    For rowIndex = 1 To 20
        If CStr(output(rowIndex, 3)) = "STOP" Then stopCount = stopCount + 1
        If CStr(output(rowIndex, 3)) = "CAUTION" Then cautionCount = cautionCount + 1
    Next rowIndex
End Sub

Private Sub WF_TrajectorySetCheck(ByRef output() As Variant, ByVal rowIndex As Long, ByVal measured As Variant, ByVal status As String, ByVal severity As String)
    output(rowIndex, 1) = measured
    output(rowIndex, 2) = status
    output(rowIndex, 3) = severity
End Sub

Private Function WF_TrajectoryTargetStatus(ByVal targets As Variant, ByVal rowIndex As Long) As String
    If CStr(targets(rowIndex, 3)) = "not_reached" Then
        WF_TrajectoryTargetStatus = "NOT REACHED"
    Else
        WF_TrajectoryTargetStatus = WF_TrajectoryDisplayEnum(CStr(targets(rowIndex, 3))) & " " & WF_TrajectoryDisplayEnum(CStr(targets(rowIndex, 4)))
    End If
End Function

Private Function WF_TrajectoryDisplayOptional(ByVal value As Variant, ByVal factor As Double) As Variant
    If IsEmpty(value) Then WF_TrajectoryDisplayOptional = Empty Else WF_TrajectoryDisplayOptional = CDbl(value) * factor
End Function

Private Function WF_TrajectoryDisplayEnum(ByVal value As String) As String
    WF_TrajectoryDisplayEnum = UCase$(Replace$(value, "_", " "))
End Function

Private Function WF_TrajectoryCaptureSnapshots() As Collection
    Dim snapshots As Collection
    Set snapshots = New Collection
    WF_TrajectorySnapshot snapshots, "Plan", "F7:M506"
    WF_TrajectorySnapshot snapshots, "Survey", "F7:Y506"
    WF_TrajectorySnapshot snapshots, "Targets", "L7:Q106"
    WF_TrajectorySnapshot snapshots, "Slide Performance", "K7:S206"
    WF_TrajectorySnapshot snapshots, "Formation Tops", "G7:K106"
    WF_TrajectorySnapshot snapshots, "Results", "A26:M525"
    WF_TrajectorySnapshot snapshots, "Results", "B6:B15"
    WF_TrajectorySnapshot snapshots, "Results", "B19:B21"
    WF_TrajectorySnapshot snapshots, "Results", "P7:P8"
    WF_TrajectorySnapshot snapshots, "Results", "P10:P14"
    WF_TrajectorySnapshot snapshots, "Checks", "B6:D25"
    WF_TrajectorySnapshot snapshots, "Summary", "B5:C9"
    WF_TrajectorySnapshot snapshots, "Summary", "B10"
    WF_TrajectorySnapshot snapshots, "Calc", "A7:R506"
    WF_TrajectorySnapshot snapshots, "Calc", "T7:BS506"
    WF_TrajectorySnapshot snapshots, "Calc", "FI7:GJ612"
    WF_TrajectorySnapshot snapshots, "Calc", "GL6:GM45"
    WF_TrajectorySnapshot snapshots, "Calc", "HB7:HT106"
    WF_TrajectorySnapshot snapshots, "Calc", "DA7:EG506"
    WF_TrajectorySnapshot snapshots, "Plan", "G5:L5"
    WF_TrajectorySnapshot snapshots, "Survey", "G5:X5"
    WF_TrajectorySnapshot snapshots, "Targets", "M5:N5"
    WF_TrajectorySnapshot snapshots, "Slide Performance", "K5:R5"
    WF_TrajectorySnapshot snapshots, "Formation Tops", "G5:H5"
    WF_TrajectorySnapshot snapshots, "Calc", "DA6:DY6"
    Set WF_TrajectoryCaptureSnapshots = snapshots
End Function

Private Sub WF_TrajectorySnapshot(ByVal snapshots As Collection, ByVal sheetName As String, ByVal address As String)
    Dim snapshot As Object
    Set snapshot = CreateObject("Scripting.Dictionary")
    snapshot.Add "sheet", sheetName
    snapshot.Add "address", address
    snapshot.Add "values", ThisWorkbook.Worksheets(sheetName).Range(address).Value2
    snapshots.Add snapshot
End Sub

Private Function WF_TrajectoryRestoreSnapshots(ByVal snapshots As Collection) As Boolean
    Dim index As Long, snapshot As Object, restoreSucceeded As Boolean
    restoreSucceeded = True
    For index = snapshots.Count To 1 Step -1
        On Error Resume Next
        Err.Clear
        Set snapshot = snapshots.Item(index)
        ThisWorkbook.Worksheets(CStr(snapshot.Item("sheet"))).Range(CStr(snapshot.Item("address"))).Value2 = snapshot.Item("values")
        If Err.Number <> 0 Then restoreSucceeded = False
        On Error GoTo 0
    Next index
    WF_TrajectoryRestoreSnapshots = restoreSucceeded
End Function

Private Function WF_TrajectorySnapshotsMatch(ByVal snapshots As Collection) As Boolean
    Dim index As Long, snapshot As Object, actualValues As Variant, expectedValues As Variant
    WF_TrajectorySnapshotsMatch = False
    On Error GoTo Mismatch
    For index = 1 To snapshots.Count
        Set snapshot = snapshots.Item(index)
        actualValues = ThisWorkbook.Worksheets(CStr(snapshot.Item("sheet"))).Range(CStr(snapshot.Item("address"))).Value2
        expectedValues = snapshot.Item("values")
        If Not WF_TrajectoryValuesMatch(actualValues, expectedValues) Then Exit Function
    Next index
    WF_TrajectorySnapshotsMatch = True
Mismatch:
End Function

Private Function WF_TrajectoryValuesMatch(ByVal actualValues As Variant, ByVal expectedValues As Variant) As Boolean
    Dim rowIndex As Long, columnIndex As Long
    On Error GoTo Mismatch
    If IsArray(expectedValues) Then
        If Not IsArray(actualValues) Then Exit Function
        For rowIndex = LBound(expectedValues, 1) To UBound(expectedValues, 1)
            For columnIndex = LBound(expectedValues, 2) To UBound(expectedValues, 2)
                If Not WF_TrajectoryValueMatches(actualValues(rowIndex, columnIndex), expectedValues(rowIndex, columnIndex)) Then Exit Function
            Next columnIndex
        Next rowIndex
    ElseIf Not WF_TrajectoryValueMatches(actualValues, expectedValues) Then
        Exit Function
    End If
    WF_TrajectoryValuesMatch = True
Mismatch:
End Function

Private Function WF_TrajectoryValueMatches(ByVal actualValue As Variant, ByVal expectedValue As Variant) As Boolean
    If IsError(actualValue) Or IsError(expectedValue) Then
        WF_TrajectoryValueMatches = IsError(actualValue) And IsError(expectedValue) And CStr(actualValue) = CStr(expectedValue)
    ElseIf IsEmpty(actualValue) Or IsEmpty(expectedValue) Then
        WF_TrajectoryValueMatches = IsEmpty(actualValue) And IsEmpty(expectedValue)
    Else
        WF_TrajectoryValueMatches = (VarType(actualValue) = VarType(expectedValue) And CStr(actualValue) = CStr(expectedValue))
    End If
End Function

Private Sub WF_PublishTrajectoryFailure(ByVal stateText As String, ByVal diagnosticPath As String, ByVal lastAcceptedValuesPreserved As Boolean)
    With ThisWorkbook.Worksheets("Results")
        .Range("P5").Value2 = WF_TRAJECTORY_EXECUTION_MODE
        If lastAcceptedValuesPreserved Then
            .Range("P6").Value2 = stateText & " — FAILED — LAST ACCEPTED VALUES PRESERVED"
        Else
            .Range("P6").Value2 = stateText & " — FAILED — ROLLBACK INCOMPLETE"
        End If
        If Len(diagnosticPath) = 0 Then
            .Range("P9").Value2 = "Diagnostics not available"
        ElseIf Len(Dir$(diagnosticPath, vbNormal)) = 0 Then
            .Range("P9").Value2 = "Diagnostics not produced; intended path: " & diagnosticPath
        Else
            .Range("P9").Value2 = diagnosticPath
        End If
    End With
End Sub

Private Sub WF_TrajectoryValidateUnitAndControlInputs()
    WF_TrajectoryRequireUnit "E5", Array("m", "ft"), "plan length unit"
    WF_TrajectoryRequireUnit "E6", Array("deg", "rad"), "plan angle unit"
    WF_TrajectoryRequireUnit "E7", Array("m", "ft"), "survey length unit"
    WF_TrajectoryRequireUnit "E8", Array("deg", "rad"), "survey angle unit"
    WF_TrajectoryRequireUnit "E9", Array("m", "ft"), "target length unit"
    WF_TrajectoryRequireUnit "E10", Array("m", "ft"), "slide length unit"
    WF_TrajectoryRequireUnit "E11", Array("m", "ft"), "formation length unit"
    WF_TrajectoryRequireUnit "H6", Array("rad/m", "deg/100ft", "deg/30m"), "DLS input unit"
    WF_TrajectoryRequireUnit "K9", Array("rad/m", "deg/100ft", "deg/30m"), "gradient input unit"
    Call WF_TrajectoryInputNumber("B13", "surface north")
    Call WF_TrajectoryInputNumber("B14", "surface east")
    Call WF_TrajectoryInputNumber("B16", "vertical section azimuth")
    Call WF_TrajectoryInputNumber("H5", "DLS operating limit")
    Call WF_TrajectoryInputNumber("N5", "low inclination threshold")
    Call WF_TrajectoryInputNumber("N6", "minimum slide length")
    Call WF_TrajectoryInputNumber("N7", "slide-yield outlier limit")
    Call WF_TrajectoryInputNumber("N9", "survey-gap warning")
End Sub

Private Sub WF_TrajectoryRequireUnit(ByVal cellAddress As String, ByVal allowedUnits As Variant, ByVal fieldName As String)
    Dim rawValue As Variant, unitValue As String, allowedUnit As Variant
    rawValue = ThisWorkbook.Worksheets("Inputs").Range(cellAddress).Value2
    If IsError(rawValue) Or IsEmpty(rawValue) Then Err.Raise vbObjectError + 8888, "WF_TrajectoryRequireUnit", "INVALID REQUEST: explicit " & fieldName & " is required"
    unitValue = LCase$(Trim$(CStr(rawValue)))
    If Len(unitValue) = 0 Then Err.Raise vbObjectError + 8888, "WF_TrajectoryRequireUnit", "INVALID REQUEST: explicit " & fieldName & " is required"
    For Each allowedUnit In allowedUnits
        If StrComp(unitValue, LCase$(CStr(allowedUnit)), vbBinaryCompare) = 0 Then Exit Sub
    Next allowedUnit
    Err.Raise vbObjectError + 8889, "WF_TrajectoryRequireUnit", "INVALID REQUEST: unsupported " & fieldName & " '" & unitValue & "'"
End Sub

Private Function WF_TrajectoryInputNumber(ByVal cellAddress As String, ByVal fieldName As String) As Double
    WF_TrajectoryInputNumber = WF_TrajectoryRequiredNumber(ThisWorkbook.Worksheets("Inputs").Range(cellAddress).Value2, fieldName)
End Function

Private Function WF_TrajectoryRequiredText(ByVal value As Variant, ByVal fieldName As String) As String
    If IsError(value) Or IsEmpty(value) Or Len(Trim$(CStr(value))) = 0 Then Err.Raise vbObjectError + 8890, "WF_TrajectoryRequiredText", "INVALID REQUEST: explicit " & fieldName & " is required"
    WF_TrajectoryRequiredText = Trim$(CStr(value))
End Function

Private Function WF_TrajectoryRequiredNumber(ByVal value As Variant, ByVal fieldName As String) As Double
    If VarType(value) = vbBoolean Then Err.Raise vbObjectError + 8891, "WF_TrajectoryRequiredNumber", "INVALID REQUEST: numeric " & fieldName & " is required"
    If IsError(value) Or IsEmpty(value) Or Len(Trim$(CStr(value))) = 0 Or Not IsNumeric(value) Then Err.Raise vbObjectError + 8891, "WF_TrajectoryRequiredNumber", "INVALID REQUEST: numeric " & fieldName & " is required"
    On Error GoTo InvalidNumber
    WF_TrajectoryRequiredNumber = CDbl(value)
    If WF_TrajectoryRequiredNumber <> WF_TrajectoryRequiredNumber Then GoTo InvalidNumber
    Exit Function
InvalidNumber:
    On Error GoTo 0
    Err.Raise vbObjectError + 8892, "WF_TrajectoryRequiredNumber", "INVALID REQUEST: non-finite " & fieldName
End Function

Private Function WF_TrajectoryOptionalInputNumber(ByVal value As Variant, ByVal unitName As String) As Double
    If IsError(value) Then Err.Raise vbObjectError + 8893, "WF_TrajectoryOptionalInputNumber", "INVALID REQUEST: optional input contains an Excel error"
    If VarType(value) = vbBoolean Then Err.Raise vbObjectError + 8893, "WF_TrajectoryOptionalInputNumber", "INVALID REQUEST: optional input must be numeric"
    If IsEmpty(value) Or Len(Trim$(CStr(value))) = 0 Then Exit Function
    WF_TrajectoryOptionalInputNumber = WF_ToSI(WF_TrajectoryRequiredNumber(value, "optional input"), unitName)
End Function

Private Function WF_TrajectoryOptionalNullableNumber(ByVal value As Variant, ByVal unitName As String, ByVal fieldName As String) As Variant
    If IsError(value) Then Err.Raise vbObjectError + 8894, "WF_TrajectoryOptionalNullableNumber", "INVALID REQUEST: " & fieldName & " contains an Excel error"
    If VarType(value) = vbBoolean Then Err.Raise vbObjectError + 8894, "WF_TrajectoryOptionalNullableNumber", "INVALID REQUEST: " & fieldName & " must be numeric"
    If IsEmpty(value) Or Len(Trim$(CStr(value))) = 0 Then
        WF_TrajectoryOptionalNullableNumber = Empty
    Else
        WF_TrajectoryOptionalNullableNumber = WF_ToSI(WF_TrajectoryRequiredNumber(value, fieldName), unitName)
    End If
End Function

Private Function WF_TrajectoryNewSet() As Object
    Dim output As Object
    Set output = CreateObject("Scripting.Dictionary")
    output.CompareMode = vbTextCompare
    Set WF_TrajectoryNewSet = output
End Function

Private Function WF_TrajectoryArraySize(ByVal countRows As Long) As Long
    If countRows > 0 Then WF_TrajectoryArraySize = countRows Else WF_TrajectoryArraySize = 1
End Function

Private Function WF_TrajectoryIsSha256(ByVal value As String) As Boolean
    Dim expression As Object
    Set expression = CreateObject("VBScript.RegExp")
    expression.Pattern = "^[0-9a-f]{64}$"
    WF_TrajectoryIsSha256 = expression.Test(value)
End Function

Private Function WF_TrajectoryIsUuid(ByVal value As String) As Boolean
    Dim expression As Object
    Set expression = CreateObject("VBScript.RegExp")
    expression.Pattern = "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    WF_TrajectoryIsUuid = expression.Test(value) And Replace$(value, "-", vbNullString) <> String$(32, "0")
End Function

Private Function WF_TrajectoryQuote(ByVal value As String) As String
    WF_TrajectoryQuote = Chr$(34) & Replace$(value, Chr$(34), Chr$(34) & Chr$(34)) & Chr$(34)
End Function

Private Function WF_TrajectoryElapsedSeconds(ByVal started As Double) As Double
    WF_TrajectoryElapsedSeconds = Timer - started
    If WF_TrajectoryElapsedSeconds < 0# Then WF_TrajectoryElapsedSeconds = WF_TrajectoryElapsedSeconds + 86400#
End Function

Private Sub WF_TrajectoryEnsureFolder(ByVal folderPath As String)
    If Len(Dir$(folderPath, vbDirectory)) = 0 Then MkDir folderPath
End Sub

Private Function WF_TrajectoryUtcTimestamp() As String
    Dim value As WF_TrajectorySystemTime
    GetSystemTime value
    WF_TrajectoryUtcTimestamp = Format$(value.Year, "0000") & "-" & Format$(value.Month, "00") & "-" & Format$(value.Day, "00") & "T" & Format$(value.Hour, "00") & ":" & Format$(value.Minute, "00") & ":" & Format$(value.Second, "00") & "." & Format$(value.Milliseconds, "000") & "Z"
End Function

Private Sub WF_TrajectoryBridgeError(ByVal message As String)
    Err.Raise vbObjectError + 8899, "WF_ParseAndValidateTrajectoryBridge", "INVALID RESULT: " & message
End Sub
