"""
ICarMeter - Ultra Low Latency HDMI USB Capture Card Viewer (1280x480)
====================================================================
Designed for testing & monitoring the Raspberry Pi Digital Dashboard
over HDMI-to-USB capture cards on Windows.

Hotkeys:
  [Q] or [ESC] : Quit
  [C]          : Cycle through video capture devices (Card / Webcam)
  [T]          : Toggle Always-On-Top window
  [S]          : Save high-res screenshot
  [F]          : Toggle Fit (Stretch to 1280x480 vs Keep Aspect Ratio)
  [H]          : Toggle HUD overlay (FPS & info)
"""

import sys
import os
import time
import datetime

try:
    import cv2
except ImportError:
    print("[!] OpenCV is required. Please install it with: pip install opencv-python")
    sys.exit(1)

TARGET_WIDTH = 1280
TARGET_HEIGHT = 480
WINDOW_NAME = "ICarMeter Capture Preview (1280x480)"

def find_available_cameras(max_tested=5):
    """Probes video capture devices on Windows via DirectShow."""
    available = []
    for i in range(max_tested):
        cap = cv2.VideoCapture(i, cv2.CAP_DSHOW)
        if cap.isOpened():
            ret, _ = cap.read()
            if ret:
                available.append(i)
            cap.release()
    return available

def main():
    print("=" * 65)
    print("  ICarMeter USB Capture Card Monitor (1280x480)")
    print("=" * 65)

    cams = find_available_cameras()
    if not cams:
        print("[!] No video capture device found.")
        print("    Please connect your USB capture card and try again.")
        cam_idx = 0
    else:
        print(f"[*] Detected video capture device indexes: {cams}")
        # Default to highest index if multiple (often capture card is plugged in after built-in webcam)
        cam_idx = cams[-1] if len(cams) > 1 else cams[0]

    print(f"[*] Opening device index {cam_idx} with DirectShow (ultra-low latency)...")
    cap = cv2.VideoCapture(cam_idx, cv2.CAP_DSHOW)

    # Configure optimal low-latency buffer & request wide resolution
    cap.set(cv2.CAP_PROP_BUFFERSIZE, 1)
    cap.set(cv2.CAP_PROP_FRAME_WIDTH, 1280)
    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 720)

    # Prefer MJPG format for USB 2.0 / 3.0 capture cards (higher FPS)
    cap.set(cv2.CAP_PROP_FOURCC, cv2.VideoWriter_fourcc(*'MJPG'))

    cv2.namedWindow(WINDOW_NAME, cv2.WINDOW_NORMAL)
    cv2.resizeWindow(WINDOW_NAME, TARGET_WIDTH, TARGET_HEIGHT)

    always_on_top = False
    fit_mode = True  # True: scale to 1280x480, False: letterbox keeping original aspect ratio
    show_hud = True
    screenshot_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "screenshots")
    os.makedirs(screenshot_dir, exist_ok=True)

    prev_time = time.time()
    fps_smooth = 0.0

    print("[*] Viewer active. Press 'H' for hotkey help, 'Q' to exit.")

    while True:
        ret, frame = cap.read()
        if not ret:
            # Display test pattern if no signal
            import numpy as np
            canvas = np.zeros((TARGET_HEIGHT, TARGET_WIDTH, 3), dtype=np.uint8)
            cv2.putText(canvas, f"NO SIGNAL - Device {cam_idx}", (TARGET_WIDTH // 2 - 200, TARGET_HEIGHT // 2),
                        cv2.FONT_HERSHEY_SIMPLEX, 0.9, (0, 0, 255), 2, cv2.LINE_AA)
            cv2.putText(canvas, "Press [C] to switch device, [Q] to quit", (TARGET_WIDTH // 2 - 220, TARGET_HEIGHT // 2 + 40),
                        cv2.FONT_HERSHEY_SIMPLEX, 0.6, (180, 180, 180), 1, cv2.LINE_AA)
        else:
            fh, fw = frame.shape[:2]

            if fit_mode:
                # Direct resize to exact 1280x480
                canvas = cv2.resize(frame, (TARGET_WIDTH, TARGET_HEIGHT), interpolation=cv2.INTER_LINEAR)
            else:
                # Letterbox / Pillarbox maintaining aspect ratio inside 1280x480
                import numpy as np
                canvas = np.zeros((TARGET_HEIGHT, TARGET_WIDTH, 3), dtype=np.uint8)
                scale = min(TARGET_WIDTH / fw, TARGET_HEIGHT / fh)
                nw, nh = int(fw * scale), int(fh * scale)
                resized = cv2.resize(frame, (nw, nh), interpolation=cv2.INTER_LINEAR)
                ox = (TARGET_WIDTH - nw) // 2
                oy = (TARGET_HEIGHT - nh) // 2
                canvas[oy:oy+nh, ox:ox+nw] = resized

        # Calculate FPS
        curr_time = time.time()
        dt = curr_time - prev_time
        prev_time = curr_time
        if dt > 0:
            current_fps = 1.0 / dt
            fps_smooth = 0.9 * fps_smooth + 0.1 * current_fps if fps_smooth > 0 else current_fps

        # Overlay HUD if enabled
        if show_hud:
            hud_text = f"Dev:{cam_idx} | 1280x480 | FPS:{fps_smooth:.1f} | Top:{'ON' if always_on_top else 'OFF'} | Fit:{'STRETCH' if fit_mode else 'RATIO'}"
            # Dark semi-transparent banner for readability
            sub_overlay = canvas[0:28, 0:420].copy()
            sub_dark = (sub_overlay * 0.4).astype('uint8')
            canvas[0:28, 0:420] = sub_dark
            cv2.putText(canvas, hud_text, (10, 19), cv2.FONT_HERSHEY_SIMPLEX, 0.48, (0, 255, 200), 1, cv2.LINE_AA)

        cv2.imshow(WINDOW_NAME, canvas)

        key = cv2.waitKey(1) & 0xFF

        if key in (ord('q'), ord('Q'), 27):  # 27 is ESC
            break
        elif key in (ord('c'), ord('C')):
            # Cycle capture device
            cams = find_available_cameras() or [0]
            next_idx = cams[(cams.index(cam_idx) + 1) % len(cams)] if cam_idx in cams else 0
            print(f"[*] Switching from device {cam_idx} to device {next_idx}...")
            cap.release()
            cam_idx = next_idx
            cap = cv2.VideoCapture(cam_idx, cv2.CAP_DSHOW)
            cap.set(cv2.CAP_PROP_BUFFERSIZE, 1)
        elif key in (ord('t'), ord('T')):
            always_on_top = not always_on_top
            flag = cv2.WINDOW_AUTOSIZE
            cv2.setWindowProperty(WINDOW_NAME, cv2.WND_PROP_TOPMOST, 1 if always_on_top else 0)
            print(f"[*] Always-on-top: {'ENABLED' if always_on_top else 'DISABLED'}")
        elif key in (ord('f'), ord('F')):
            fit_mode = not fit_mode
            print(f"[*] Mode: {'Direct 1280x480 stretch' if fit_mode else 'Letterbox aspect ratio'}")
        elif key in (ord('h'), ord('H')):
            show_hud = not show_hud
        elif key in (ord('s'), ord('S')):
            timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
            shot_path = os.path.join(screenshot_dir, f"snap_{timestamp}.png")
            cv2.imwrite(shot_path, canvas)
            print(f"[+] Saved screenshot: {shot_path}")

    cap.release()
    cv2.destroyAllWindows()
    print("[*] Viewer closed.")

if __name__ == "__main__":
    main()
