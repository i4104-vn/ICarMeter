#!/usr/bin/env bash
# ==============================================================================
# Car Digital Dashboard - Linux Framebuffer (tiny-skia)
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
echo -e "${PURPLE}       CAR DIGITAL DASHBOARD - RASPBERRY PI EMBEDDED UI             ${NC}"
echo -e "${CYAN}====================================================================${NC}"
echo -e "  * Kích thước táp-lô : ${GREEN}1280 x 480 pixels${NC} (Native 1:1 Aspect Ratio)"
echo -e "  * Render Pipeline   : ${YELLOW}tiny-skia direct framebuffer (/dev/fb0)${NC}"
echo -e "  * Phím tắt          : [M] Đổi chế độ TPMS / Speed Chart | [A] Demo GPS"
echo -e "  * Thoát chương trình: ${GREEN}Bấm [q / Q / Esc / Ctrl+C]${NC}"
echo -e "${CYAN}====================================================================${NC}"

# Detect architecture
ARCH=$(uname -m)
if [[ "$ARCH" == "aarch64" ]]; then
    TARGET=""
    BIN_PATH="$SCRIPT_DIR/target/release/icarmeter"
else
    TARGET="--target x86_64-unknown-linux-gnu"
    BIN_PATH="$SCRIPT_DIR/target/x86_64-unknown-linux-gnu/release/icarmeter"
fi

echo -e "${YELLOW}[1/2] Đang biên dịch release...${NC}"
cargo build --release $TARGET

if [[ ! -f "$BIN_PATH" ]]; then
    BIN_PATH="$SCRIPT_DIR/target/release/icarmeter"
fi

if [[ ! -f "$BIN_PATH" ]]; then
    echo -e "\033[0;31mKhông tìm thấy file thực thi: $BIN_PATH\033[0m"
    exit 1
fi

FB_DEV="/dev/fb0"
if [[ ! -e "$FB_DEV" ]]; then
    echo -e "\033[1;33mCảnh báo: Không tìm thấy $FB_DEV (nếu chạy trên PC/môi trường dev không có framebuffer ảo).\033[0m"
fi

echo -e "${GREEN}[2/2] Khởi chạy Car Dashboard...${NC}"
sudo "$BIN_PATH" || "$BIN_PATH"
