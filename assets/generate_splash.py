#!/usr/bin/env python3
"""
Generator for Fluffy Cat Sipping Cup with Breathing Motion and Floating Smoke.
Smoke puffs are positioned EXCLUSIVELY on the LEFT side matching the user's reference image
(at the left cheek/cup area) and smoothly rise and dissipate.
Canvas: 1280x480, Pure Solid Black (#000000).
"""

import math
import os
import shutil
from PIL import Image, ImageChops, ImageFilter

LOCAL_IMAGE_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "image.png")

CANVAS_W = 1280
CANVAS_H = 480
TOTAL_FRAMES = 36

def clean_scratch_files():
    for f in [
        "cat_test.png", "eye_test_left.png", "eye_test_right.png",
        "left_eye_crop.png", "right_eye_crop.png", "test_badge.png",
        "test_black_bg.png", "test_canvas_preview.png", "test_cat_cutout.png",
        "test_splash_composite.png", "test_scene.png", "test_smoke_puff.png",
        "test_sprite.png", "sample_region.png", "extracted_smoke.png"
    ]:
        if os.path.exists(f):
            try:
                os.remove(f)
            except Exception:
                pass

def extract_alpha_sprite(im_crop, threshold=15):
    """Converts a black-background crop into an RGBA sprite with smooth alpha."""
    w, h = im_crop.size
    rgba = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    src_pixels = im_crop.load()
    dest_pixels = rgba.load()

    for y in range(h):
        for x in range(w):
            r, g, b = src_pixels[x, y][:3]
            brightness = max(r, g, b)
            if brightness > threshold:
                # Smooth curve for alpha
                alpha = int(min(255, (brightness - threshold) * 255.0 / (255.0 - threshold)))
                dest_pixels[x, y] = (255, 255, 255, alpha)

    return rgba

def main():
    clean_scratch_files()

    src = Image.open(LOCAL_IMAGE_PATH).convert('RGB')
    sw, sh = src.size

    # The user image is 464x490:
    # Smoke region is on the LEFT: x in [20, 105], y in [100, 220]
    # Cat body + tail is for x >= 105, y in [100, 340]
    # Let's cleanly separate the cat body from the left smoke
    # so we don't have static smoke freezing while animated smoke floats!
    src_pixels = src.load()

    cat_rgba = Image.new("RGBA", (sw, sh), (0, 0, 0, 0))
    cat_pixels = cat_rgba.load()

    smoke_rgba = Image.new("RGBA", (sw, sh), (0, 0, 0, 0))
    smoke_pixels = smoke_rgba.load()

    smoke_split_x = int(sw * 0.18)
    smoke_split_y = int(sh * 0.46)
    wm_split_x = int(sw * 0.85)
    wm_split_y = int(sh * 0.90)

    for y in range(sh):
        for x in range(sw):
            if y >= wm_split_y and x >= wm_split_x:
                # Remove watermark noise at the bottom right
                continue

            r, g, b = src_pixels[x, y]
            brightness = max(r, g, b)
            if brightness <= 15:
                continue

            alpha = int(min(255, (brightness - 15) * 255.0 / 240.0))

            # Left smoke region vs Cat body
            if x < smoke_split_x and y < smoke_split_y:
                smoke_pixels[x, y] = (r, g, b, alpha)
            else:
                cat_pixels[x, y] = (r, g, b, alpha)

    # Crop cat body to tight bounding box
    cat_bbox = cat_rgba.getbbox()
    if cat_bbox is None:
        cat_bbox = (0, 0, sw, sh)
    cropped_cat = cat_rgba.crop(cat_bbox)
    cw, ch = cropped_cat.size

    # Extract the smoke puffs dynamically from the smoke layer
    smoke_bbox = smoke_rgba.getbbox()
    if smoke_bbox is None:
        smoke_bbox = (0, 0, smoke_split_x, smoke_split_y)

    mid_sy = (smoke_bbox[1] + smoke_bbox[3]) // 2
    puff_upper_raw = smoke_rgba.crop((smoke_bbox[0], smoke_bbox[1], smoke_bbox[2], mid_sy))
    puff_upper_bbox = puff_upper_raw.getbbox()
    puff_upper = puff_upper_raw.crop(puff_upper_bbox) if puff_upper_bbox else puff_upper_raw

    puff_lower_raw = smoke_rgba.crop((smoke_bbox[0], mid_sy, smoke_bbox[2], smoke_bbox[3]))
    puff_lower_bbox = puff_lower_raw.getbbox()
    puff_lower = puff_lower_raw.crop(puff_lower_bbox) if puff_lower_bbox else puff_lower_raw

    # Scale uniformly to height 260 so the image has comfortable margins on the 1280x480 canvas.
    target_h = 260
    base_scale = target_h / float(ch)
    target_w = int(round(cw * base_scale))

    base_x = (CANVAS_W - target_w) // 2
    base_y = (CANVAS_H - target_h) // 2

    # In cat-relative coordinates, the smoke in the reference image
    # is located on the LEFT side of the cat:
    smoke_center_x = (smoke_bbox[0] + smoke_bbox[2]) / 2.0
    smoke_lower_y = (mid_sy + smoke_bbox[3]) / 2.0

    ref_dx = int((smoke_center_x - cat_bbox[0]) * base_scale)
    ref_lower_dy = int((smoke_lower_y - cat_bbox[1]) * base_scale)

    # Prepare smoke sprites
    puff_sprites = [puff_lower, puff_upper]

    frames = []

    smoke_scale_factor = target_h / 320.0

    for f_idx in range(TOTAL_FRAMES):
        phase = f_idx / float(TOTAL_FRAMES)

        # Breathing wave (Cosine ease-in-out)
        b = 0.5 - 0.5 * math.cos(phase * 2.0 * math.pi)
        dy = -5.0 * b
        breathe_scale = 1.0 + 0.042 * b

        fw = int(round(target_w * breathe_scale))
        fh = int(round(target_h * breathe_scale))

        scaled_cat = cropped_cat.resize((fw, fh), Image.Resampling.LANCZOS)

        cat_px = int(round((CANVAS_W - fw) / 2.0))
        cat_py = int(round(base_y + dy + (target_h - fh)))

        # Create pure solid black canvas (#000000)
        canvas = Image.new("RGBA", (CANVAS_W, CANVAS_H), (0, 0, 0, 255))

        # Paste the cat body
        canvas.paste(scaled_cat, (cat_px, cat_py), mask=scaled_cat)

        # ----------------------------------------------------------------------
        # Smoke Puffs on the LEFT side ONLY (3 staggered floating puffs)
        # Position: Rising along the left cheek/ear/cup area as in user's image
        # NO SMOKE ON THE RIGHT!
        # ----------------------------------------------------------------------
        smoke_base_x = cat_px + ref_dx + int(15 * smoke_scale_factor)
        smoke_base_y = cat_py + ref_lower_dy + int(20 * breathe_scale * smoke_scale_factor) - int(15 * smoke_scale_factor)
        rise_distance = int(120 * breathe_scale * smoke_scale_factor)

        for k in range(3):
            p = (phase + k / 3.0) % 1.0

            # Y-position: rises from smoke_base_y upwards by rise_distance
            puff_y = smoke_base_y - (p * rise_distance)

            # X-position: gentle wavy drift (swaying slightly left then back)
            puff_x = smoke_base_x - math.sin(p * math.pi * 1.5) * (14.0 * breathe_scale * smoke_scale_factor)

            # Scale: starts around 0.85, grows slightly to 1.15, scaled proportionally
            puff_scale = breathe_scale * (0.85 + 0.30 * p) * smoke_scale_factor

            # Rotation: gentle tilt
            puff_angle = -12.0 + p * 24.0

            # Opacity curve (fade-in, hold, fade-out)
            if p < 0.20:
                op = (p / 0.20)
            elif p < 0.65:
                op = 1.0
            else:
                op = max(0.0, 1.0 - (p - 0.65) / 0.35)

            if op > 0.02:
                # Select sprite (alternating between lower and upper puff shapes)
                raw_sprite = puff_sprites[k % 2]
                pw = max(1, int(round(raw_sprite.width * puff_scale)))
                ph = max(1, int(round(raw_sprite.height * puff_scale)))

                sp = raw_sprite.resize((pw, ph), Image.Resampling.LANCZOS)
                if abs(puff_angle) > 1.0:
                    sp = sp.rotate(puff_angle, resample=Image.Resampling.BICUBIC, expand=True)

                # Apply opacity
                if op < 0.98:
                    r, g, b, a = sp.split()
                    a = a.point(lambda v: int(v * op))
                    sp.putalpha(a)

                # Add soft glow
                glow = sp.filter(ImageFilter.GaussianBlur(radius=2))
                combined_sprite = Image.alpha_composite(glow, sp)

                sp_x = int(round(puff_x - combined_sprite.width / 2.0))
                sp_y = int(round(puff_y - combined_sprite.height / 2.0))

                canvas.paste(combined_sprite, (sp_x, sp_y), mask=combined_sprite)

        frames.append(canvas.convert("RGB"))

    # Save outputs
    theme_dir = os.path.join("pi-image", "stage-icarmeter", "01-bootscreen", "files", "icarmeter")
    os.makedirs(theme_dir, exist_ok=True)

    for old_file in os.listdir(theme_dir):
        if old_file.startswith("frame-") and old_file.endswith(".png"):
            os.remove(os.path.join(theme_dir, old_file))

    for idx, frame in enumerate(frames):
        frame.save(os.path.join(theme_dir, f"frame-{idx}.png"), "PNG")

    frames[0].save(os.path.join(theme_dir, "splash.png"), "PNG")
    
    # Save a preview GIF in assets/ for testing
    preview_gif = os.path.join(os.path.dirname(os.path.abspath(__file__)), "splash.gif")
    frames[0].save(
        preview_gif,
        save_all=True,
        append_images=frames[1:],
        duration=50,
        loop=0,
        optimize=True
    )

    print("Complete: Generated 36 frames and animated GIF with smoke on the LEFT side.")

if __name__ == "__main__":
    main()
