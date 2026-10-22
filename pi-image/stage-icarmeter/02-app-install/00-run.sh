#!/bin/bash -e
# ==============================================================================
# Host-side: Install ICarMeter binary, configuration, and boot integration
# ==============================================================================

APP_DIR="${ROOTFS_DIR}/opt/icarmeter"
install -d -m 755 "${APP_DIR}"
if [ -f files/config.toml ]; then
install -m 666 files/config.toml "${APP_DIR}/config.toml"
fi
install -m 755 files/icarmeter-bt-init.sh "${APP_DIR}/icarmeter-bt-init.sh"
install -d -m 755 "${APP_DIR}/licenses"
install -m 644 files/MAPLIBRE-NATIVE-LICENSE.txt "${APP_DIR}/licenses/MAPLIBRE-NATIVE-LICENSE.txt"
install -d -m 755 "${ROOTFS_DIR}/etc/profile.d"
install -m 644 files/icarmeter-dashboard.sh "${ROOTFS_DIR}/etc/profile.d/icarmeter-dashboard.sh"
install -m 644 files/icarmeter-bluetooth.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-bluetooth.service"
install -m 644 files/icarmeter-bt-agent.service "${ROOTFS_DIR}/etc/systemd/system/icarmeter-bt-agent.service"


# Install the local MapLibre style and allow optional custom map resources.
if [ -d files/maps ]; then
    install -d -m 755 "${APP_DIR}/maps"
    cp -a files/maps/. "${APP_DIR}/maps/"
fi

# BBBike publishes an OSM Shortbread vector MBTiles extract for all of Vietnam.
# Download it while building the image so the running dashboard needs no network.
MAP_ARCHIVE_URL="https://data.bbbike.org/osm/mbtiles/region/asia/vietnam/vietnam.osm.mbtiles-shortbread.zip"
MAP_TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "${MAP_TEMP_DIR}"' EXIT
curl --fail --location --retry 3 --retry-delay 2 \
    "${MAP_ARCHIVE_URL}" -o "${MAP_TEMP_DIR}/vietnam-shortbread.zip"
MAP_ARCHIVE_ENTRY="$(unzip -Z1 "${MAP_TEMP_DIR}/vietnam-shortbread.zip" | awk 'tolower($0) ~ /\.mbtiles$/ { print; exit }')"
if [ -z "${MAP_ARCHIVE_ENTRY}" ]; then
    echo "Vietnam offline map archive did not contain an MBTiles file" >&2
    exit 1
fi
unzip -p "${MAP_TEMP_DIR}/vietnam-shortbread.zip" "${MAP_ARCHIVE_ENTRY}" \
    > "${MAP_TEMP_DIR}/vietnam.mbtiles"
if ! file "${MAP_TEMP_DIR}/vietnam.mbtiles" | grep -q "SQLite 3.x database"; then
    echo "Downloaded Vietnam map is not a valid MBTiles database" >&2
    exit 1
fi
if command -v python3 >/dev/null 2>&1; then
    python3 - "${MAP_TEMP_DIR}/vietnam.mbtiles" <<'PY'
import sqlite3
import sys

database = sqlite3.connect(f"file:{sys.argv[1]}?mode=ro", uri=True)
tables = {row[0] for row in database.execute("SELECT name FROM sqlite_master WHERE type='table'")}
if not {"metadata", "tiles"}.issubset(tables):
    raise SystemExit("Vietnam map is missing MBTiles metadata or tile tables")
metadata = dict(database.execute("SELECT name, value FROM metadata"))
if metadata.get("format") not in {"pbf", "mvt"}:
    raise SystemExit(f"Expected vector MBTiles, found format={metadata.get('format')!r}")
if database.execute("SELECT 1 FROM tiles LIMIT 1").fetchone() is None:
    raise SystemExit("Vietnam vector MBTiles contains no tiles")
database.close()
PY
else
    echo "WARNING: python3 not found on host, skipping deep MBTiles table validation"
fi
install -m 644 "${MAP_TEMP_DIR}/vietnam.mbtiles" "${APP_DIR}/maps/vietnam.mbtiles"

if [ -f files/icarmeter ]; then
    echo "Found pre-compiled icarmeter binary. Installing into rootfs..."
    install -m 755 files/icarmeter "${APP_DIR}/icarmeter"
else
    echo "ERROR: Pre-compiled binary 'files/icarmeter' not found!" >&2
    echo "Please build the Rust binary first (e.g. via cargo build --release --target aarch64-unknown-linux-gnu)" >&2
    echo "and place it at 'stage-icarmeter/02-app-install/files/icarmeter' before building the OS image." >&2
    exit 1
fi
