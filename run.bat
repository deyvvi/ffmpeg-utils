@echo off
cd /d "%~dp0"
if exist "src-tauri\target\release\ffmpeg-utils.exe" (
    start "" "src-tauri\target\release\ffmpeg-utils.exe"
    exit /b 0
)
echo Release build not found. Starting development mode...
call npm run tauri dev
