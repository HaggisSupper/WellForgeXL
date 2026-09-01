@echo off
setlocal EnableExtensions

rem WellForge HTML UI launcher. Run this file from Explorer or a terminal.
set "UI_ROOT=%~dp0.."
set "PORT=%WELLFORGE_UI_PORT%"
if not defined PORT set "PORT=8765"

where py.exe >nul 2>&1
if not errorlevel 1 (
  set "PYTHON=py.exe"
  set "PYTHON_ARGS=-3"
) else (
  where python.exe >nul 2>&1
  if errorlevel 1 (
    echo Python 3 was not found. Install Python 3 or add it to PATH.
    exit /b 1
  )
  set "PYTHON=python.exe"
  set "PYTHON_ARGS="
)

echo Starting WellForge UI server on http://127.0.0.1:%PORT% ...
start "WellForge UI server" /min "%PYTHON%" %PYTHON_ARGS% -m http.server %PORT% --directory "%UI_ROOT%"

rem Give the server a moment to bind before opening the browser.
ping 127.0.0.1 -n 2 >nul
start "" "http://127.0.0.1:%PORT%/HTML%%20UI/"
echo WellForge UI opened. Close the "WellForge UI server" window to stop it.
endlocal
