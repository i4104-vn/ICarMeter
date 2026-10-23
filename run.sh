#!/usr/bin/env bash
# ==============================================================================
# Car Digital Dashboard - Raspberry Pi OS with Slint LinuxKMS
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${CYAN}====================================================================${NC}"
echo -e "${PURPLE}       CAR DIGITAL DASHBOARD - RASPBERRY PI OS EMBEDDED UI          ${NC}"
echo -e "${CYAN}====================================================================${NC}"
echo -e "  * Kích thước táp-lô : ${GREEN}1280 x 480 pixels${NC} (7 inch)"
echo -e "  * Nền tảng mục tiêu : ${YELLOW}Raspberry Pi OS (DRM/KMS)${NC}"
echo -e "  * Giao diện         : ${YELLOW}Slint LinuxKMS${NC}"
echo -e "  * Thoát chương trình: ${GREEN}Bấm [q / Q / Ctrl+C]${NC}"
echo -e "${CYAN}====================================================================${NC}"

DRM_DEV="/dev/dri/card0"
if [[ ! -e "$DRM_DEV" ]]; then
    echo -e "\033[0;31mLỗi: Không tìm thấy $DRM_DEV. Hãy bật driver đồ họa KMS (vc4-kms-v3d).\033[0m"
    exit 1
fi

echo -e "${YELLOW}[1/2] Đang biên dịch release...${NC}"
cargo build --release --target aarch64-unknown-linux-gnu

BIN_PATH="$SCRIPT_DIR/target/aarch64-unknown-linux-gnu/release/icarmeter"
if [[ ! -f "$BIN_PATH" ]]; then
    echo -e "\033[0;31mKhông tìm thấy file thực thi: $BIN_PATH\033[0m"
    exit 1
fi

echo -e "${GREEN}[2/2] Khởi chạy Car Dashboard qua $DRM_DEV...${NC}"
"$BIN_PATH"
