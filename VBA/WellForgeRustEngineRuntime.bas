Attribute VB_Name = "WellForgeRustEngineRuntime"
Option Explicit

Private Const WF_RUST_RUNTIME_ERROR As Long = vbObjectError + 8900

Public Function WF_RustQuote(ByVal Value As String) As String
    WF_RustQuote = Chr$(34) & Replace$(Value, Chr$(34), Chr$(34) & Chr$(34)) & Chr$(34)
End Function

Public Function WF_RustElapsedSeconds(ByVal Started As Double) As Double
    WF_RustElapsedSeconds = Timer - Started
    If WF_RustElapsedSeconds < 0# Then WF_RustElapsedSeconds = WF_RustElapsedSeconds + 86400#
End Function

Public Function WF_RustExecBounded(ByVal CommandLine As String, ByVal TimeoutSeconds As Double, ByRef StdOutText As String, ByRef StdErrText As String) As Long
    Dim shell As Object, process As Object, started As Double
    On Error GoTo Failed
    Set shell = CreateObject("WScript.Shell")
    Set process = shell.Exec(CommandLine)
    started = Timer
    Do While process.Status = 0
        DoEvents
        If WF_RustElapsedSeconds(started) > TimeoutSeconds Then
            process.Terminate
            Err.Raise WF_RUST_RUNTIME_ERROR + 1, "WF_RustExecBounded", "ENGINE TIMEOUT"
        End If
    Loop
    StdOutText = process.StdOut.ReadAll
    StdErrText = process.StdErr.ReadAll
    WF_RustExecBounded = process.ExitCode
    Exit Function
Failed:
    If Err.Number = WF_RUST_RUNTIME_ERROR + 1 Then Err.Raise Err.Number, Err.Source, Err.Description
    Err.Raise WF_RUST_RUNTIME_ERROR, "WF_RustExecBounded", Err.Description
End Function

Private Function WF_RustReadProcessExitCode(ByVal ExitCodePath As String) As Long
    Dim textValue As String
    If Len(Dir$(ExitCodePath, vbNormal)) = 0 Then WF_RustReadProcessExitCode = 1: Exit Function
    textValue = Trim$(ReadUtf8File(ExitCodePath))
    If IsNumeric(textValue) Then WF_RustReadProcessExitCode = CLng(textValue) Else WF_RustReadProcessExitCode = 1
End Function

Public Function WF_RustFileSha256(ByVal FilePath As String) As String
    Dim outputText As String, normalizedOutput As String, errorText As String, exitCode As Long
    Dim line As Variant, candidate As String
    exitCode = WF_RustExecBounded("certutil.exe -hashfile " & WF_RustQuote(FilePath) & " SHA256", _
        30#, outputText, errorText)
    If exitCode <> 0 Then Err.Raise WF_RUST_RUNTIME_ERROR + 2, "WF_RustFileSha256", "Unable to hash engine: " & Trim$(errorText)
    ' certutil emits a header and a spaced digest; accept only the canonical 64-hex line.
    normalizedOutput = Replace$(outputText, vbCr, vbNullString)
    For Each line In Split(normalizedOutput, vbLf)
        candidate = Replace$(Trim$(CStr(line)), " ", vbNullString)
        If WF_RustIsSha256(candidate) Then
            WF_RustFileSha256 = LCase$(candidate)
            Exit Function
        End If
    Next line
    Err.Raise WF_RUST_RUNTIME_ERROR + 2, "WF_RustFileSha256", "Unable to parse SHA256 digest for engine: " & FilePath
End Function

Public Function WF_RustIsSha256(ByVal Value As String) As Boolean
    Dim i As Long, ch As String
    Value = LCase$(Trim$(Value))
    If Len(Value) <> 64 Then Exit Function
    For i = 1 To Len(Value)
        ch = Mid$(Value, i, 1)
        If InStr(1, "0123456789abcdef", ch, vbBinaryCompare) = 0 Then Exit Function
    Next i
    WF_RustIsSha256 = True
End Function

Public Sub WF_RustEnsureFolder(ByVal FolderPath As String)
    If Len(Dir$(FolderPath, vbDirectory)) = 0 Then MkDir FolderPath
End Sub

Public Function WF_RustFreshRunDirectory(ByVal EngineName As String, ByVal AnalysisId As String) As String
    Dim parentPath As String, runPath As String
    parentPath = Environ$("TEMP") & Application.PathSeparator & EngineName
    WF_RustEnsureFolder parentPath
    runPath = parentPath & Application.PathSeparator & Replace$(AnalysisId, "-", "") & "-" & Format$(CLng(Timer * 1000#), "000000000")
    WF_RustEnsureFolder runPath
    WF_RustFreshRunDirectory = runPath
End Function

Public Function WF_RustNumber(ByVal Value As Variant) As Double
    If IsError(Value) Or IsEmpty(Value) Or Not IsNumeric(Value) Then Err.Raise WF_RUST_RUNTIME_ERROR + 3, "WF_RustNumber", "NON-NUMERIC ENGINE VALUE"
    WF_RustNumber = CDbl(Value)
    If WF_RustNumber <> WF_RustNumber Then Err.Raise WF_RUST_RUNTIME_ERROR + 4, "WF_RustNumber", "NON-FINITE ENGINE VALUE"
End Function
