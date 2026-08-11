@echo off
setlocal

cd /d "%~dp0"
set "CHATTY_ART_BASE_PATH=%~dp0"

if not exist "chatty-art.exe" (
    echo Missing chatty-art.exe in %~dp0
    pause
    exit /b 1
)

start "" "chatty-art.exe"
