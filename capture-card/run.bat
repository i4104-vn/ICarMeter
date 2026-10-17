@echo off
title ICarMeter USB Capture Viewer (1280x480)
cd /d "%~dp0"

echo ========================================================
echo   ICarMeter - Raspberry Pi Screen Viewer via Capture Card
echo ========================================================

python -c "import cv2" >nul 2>&1
if %errorlevel% neq 0 (
    echo [*] Installing required opencv-python package...
    pip install opencv-python numpy
)

python view.py
if %errorlevel% neq 0 (
    echo [!] An error occurred while running view.py.
    pause
)
