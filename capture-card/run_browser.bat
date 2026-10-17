@echo off
title ICarMeter Web Viewer (1280x480)
cd /d "%~dp0"

:: Try launching in Edge App Mode (clean borderless window without URL bar)
start "" msedge --app="file:///%~dp0viewer.html" --window-size=1296,525 2>nul
if %errorlevel% neq 0 (
    :: Fallback to default browser
    start "" "viewer.html"
)
