#!/usr/bin/env python3
"""
Downloads complete offline OpenStreetMap raster tiles for entire Vietnam at fixed Zoom 14.
Provides street-level driving navigation with a 1-2 km radius around the vehicle:
- Single fixed zoom level: Zoom 14 (~1.75 km horizontal & vertical radius around vehicle)
- Full Nationwide Coverage: 100% of Vietnam's landmass, all 63 provinces, national highways & islands
- Storage path: {out_dir}/14/{x}/{y}.png
"""

import concurrent.futures
import math
import os
import random
import sys
import time
import urllib.request

USER_AGENT = "ICarMeter-Dashboard/1.0 (Vietnam Offline GPS Tile Provisioning; I4104.vn@gmail.com)"
FIXED_ZOOM = 14

# Generous regional slices covering 100% of Vietnam's territory and islands without ocean waste
VN_REGIONAL_BOUNDS = [
    (8.3, 9.5, 103.7, 105.8),   # Cà Mau, Bạc Liêu, Kiên Giang, Phú Quốc
    (9.3, 10.6, 104.7, 106.9),  # Miền Tây Nam Bộ (Cần Thơ, Hậu Giang, Sóc Trăng, Tiền Giang, Bến Tre, Vĩnh Long, Trà Vinh, An Giang, Đồng Tháp)
    (10.3, 11.9, 105.7, 107.7), # Đông Nam Bộ (TP.HCM, Đồng Nai, Bình Dương, BR-Vũng Tàu, Tây Ninh, Bình Phước, Long An)
    (10.7, 12.3, 107.4, 109.4), # Duyên hải Nam Trung Bộ & Cao nguyên Nam (Bình Thuận, Ninh Thuận, Lâm Đồng - Đà Lạt)
    (11.8, 13.5, 107.2, 109.5), # Đắk Nông, Đắk Lắk (Buôn Ma Thuột), Khánh Hòa (Nha Trang), Phú Yên
    (13.4, 15.0, 107.2, 109.3), # Gia Lai (Pleiku), Kon Tum, Bình Định (Quy Nhơn), Quảng Ngãi
    (14.8, 16.6, 107.1, 108.7), # Quảng Nam, Đà Nẵng, Thừa Thiên Huế
    (16.4, 18.2, 105.8, 107.3), # Quảng Trị, Quảng Bình
    (18.0, 20.3, 104.7, 106.3), # Hà Tĩnh, Nghệ An (Vinh), Thanh Hóa
    (19.8, 21.8, 105.7, 108.2), # Duyên hải Bắc Bộ (Ninh Bình, Nam Định, Thái Bình, Hải Phòng, Quảng Ninh - Vịnh Hạ Long)
    (20.4, 23.5, 105.0, 107.1), # Đồng bằng sông Hồng & Đông Bắc Bộ (Hà Nội, Bắc Ninh, Hưng Yên, Hải Dương, Vĩnh Phúc, Thái Nguyên, Lạng Sơn, Cao Bằng, Hà Giang)
    (20.3, 23.5, 102.1, 105.5), # Tây Bắc Bộ (Hòa Bình, Sơn La, Điện Biên, Lai Châu, Lào Cai - Sa Pa, Yên Bái)
    (8.5, 8.8, 106.5, 106.7),   # Côn Đảo
    (10.4, 10.7, 108.8, 109.1), # Phú Quý
]

SERVERS = [
    "https://tile.openstreetmap.org",
    "https://a.tile.openstreetmap.org",
    "https://b.tile.openstreetmap.org",
    "https://c.tile.openstreetmap.org",
    "https://tile.openstreetmap.de",
]

def deg2num(lat_deg, lon_deg, zoom):
    lat_rad = math.radians(lat_deg)
    n = 2.0 ** zoom
    xtile = int((lon_deg + 180.0) / 360.0 * n)
    ytile = int((1.0 - math.asinh(math.tan(lat_rad)) / math.pi) / 2.0 * n)
    return max(0, min(int(n) - 1, xtile)), max(0, min(int(n) - 1, ytile))

def fetch_single_tile(item):
    z, x, y, out_dir = item
    tile_dir = os.path.join(out_dir, str(z), str(x))
    tile_path = os.path.join(tile_dir, f"{y}.png")

    if os.path.exists(tile_path) and os.path.getsize(tile_path) > 300:
        return True

    os.makedirs(tile_dir, exist_ok=True)

    for attempt in range(4):
        server = random.choice(SERVERS)
        url = f"{server}/{z}/{x}/{y}.png"
        try:
            req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
            with urllib.request.urlopen(req, timeout=10) as resp:
                if resp.status == 200:
                    data = resp.read()
                    if len(data) > 200:
                        tmp_path = f"{tile_path}.tmp"
                        with open(tmp_path, "wb") as f:
                            f.write(data)
                        os.replace(tmp_path, tile_path)
                        return True
        except Exception:
            time.sleep(0.15 * (attempt + 1))
    return False

def collect_vietnam_tiles(zoom):
    tiles = set()
    for lat_min, lat_max, lon_min, lon_max in VN_REGIONAL_BOUNDS:
        x_min, y_max = deg2num(lat_min, lon_min, zoom)
        x_max, y_min = deg2num(lat_max, lon_max, zoom)
        for x in range(min(x_min, x_max), max(x_min, x_max) + 1):
            for y in range(min(y_min, y_max), max(y_min, y_max) + 1):
                tiles.add((zoom, x, y))
    return sorted(tiles)

def main():
    out_dir = sys.argv[1] if len(sys.argv) > 1 else "/opt/icarmeter/maps/tiles"
    zoom = int(sys.argv[2]) if len(sys.argv) > 2 else FIXED_ZOOM
    os.makedirs(out_dir, exist_ok=True)

    print(f"[Map] Initializing Vietnam offline map download...")
    print(f"[Map] Fixed Zoom Level: {zoom} (1-2 km radius around vehicle)")
    print(f"[Map] Destination: {out_dir}")

    tile_coords = collect_vietnam_tiles(zoom)
    total = len(tile_coords)
    print(f"[Map] Total tiles required to cover 100% Vietnam landmass at Zoom {zoom}: {total}")

    # Check which tiles are already downloaded on disk
    to_download = []
    already_done = 0
    for z, x, y in tile_coords:
        p = os.path.join(out_dir, str(z), str(x), f"{y}.png")
        if os.path.exists(p) and os.path.getsize(p) > 300:
            already_done += 1
        else:
            to_download.append((z, x, y, out_dir))

    print(f"[Map] Existing cached tiles: {already_done}/{total}")
    print(f"[Map] Tiles to download: {len(to_download)}")

    if not to_download:
        print("[Map] 100% of Vietnam map tiles are already downloaded and verified!")
        return

    workers = min(8, (os.cpu_count() or 4) * 2)
    print(f"[Map] Downloading using {workers} parallel threads across OSM mirror servers...")
    
    start_time = time.time()
    completed = already_done
    failed = 0

    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as executor:
        futures = {executor.submit(fetch_single_tile, item): item for item in to_download}
        for future in concurrent.futures.as_completed(futures):
            success = future.result()
            completed += 1
            if not success:
                failed += 1
            if completed % 100 == 0 or completed == total:
                elapsed = time.time() - start_time
                pct = (completed / total) * 100.0
                rate = (completed - already_done) / max(0.1, elapsed)
                print(f"[Map] Progress: {completed}/{total} ({pct:.1f}%) | {rate:.1f} tiles/s | Failed: {failed}")

    print(f"[Map] Finished downloading offline Vietnam map tiles into {out_dir}")

if __name__ == "__main__":
    main()
