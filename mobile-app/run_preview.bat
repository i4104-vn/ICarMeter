@echo off
title ICarMeter Mobile App Preview
cd /d "%~dp0www"
echo ===================================================
echo   ICarMeter Companion App - Flat Mobile Preview
echo   Open in browser: http://localhost:3000
echo ===================================================
start "" msedge --app="http://localhost:3000" --window-size=430,860 2>nul
if %errorlevel% neq 0 (
    start "" "http://localhost:3000"
)
npx -y serve -p 3000 .
pause
