@echo off
setlocal
cd /d "%~dp0"
if not exist "node_modules" (
    echo Dependencies are missing. Run install.bat first.
    pause
    exit /b 1
)
call npm run tauri build
if errorlevel 1 (
    echo Build failed.
    pause
    exit /b 1
)
echo.
echo Installer: src-tauri\target\release\bundle\nsis
pause
exit /b 0
