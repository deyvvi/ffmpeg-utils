@echo off
setlocal
cd /d "%~dp0"
where node >nul 2>nul
if errorlevel 1 (
    echo Node.js was not found. Install Node.js 20 or newer first.
    pause
    exit /b 1
)
echo Installing frontend and Tauri dependencies...
call npm install
if errorlevel 1 (
    echo npm install failed.
    pause
    exit /b 1
)
echo.
echo Done. Run run.bat to start the app or build.bat to create an installer.
pause
exit /b 0
