#!/usr/bin/env bash
# ==============================================================================
# Car Digital Dashboard - Khởi chạy trực tiếp trên Raspberry Pi OS (Debian)
# Xuất hình qua Linux Framebuffer (/dev/fb0) trên Console TTY thuần túy
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
echo -e "  * Nền tảng mục tiêu : ${YELLOW}Raspberry Pi OS (Debian) Console TTY${NC}"
echo -e "  * Driver hiển thị   : ${YELLOW}Linux Framebuffer (/dev/fb0)${NC}"
echo -e "  * Thoát chương trình: ${GREEN}Bấm [q / Q / Esc / Ctrl+C]${NC}"
echo -e "${CYAN}====================================================================${NC}"

FB_DEV="/dev/fb0"
if [[ ! -e "$FB_DEV" ]]; then
    echo -e "\033[0;31mLỗi: Không tìm thấy $FB_DEV. Hãy đảm bảo driver đồ họa (vc4-kms-v3d) đã được bật trong /boot/firmware/config.txt.\033[0m"
    exit 1
fi

echo -e "${YELLOW}[1/2] Đang biên dịch release...${NC}"
cargo build --release

BIN_PATH="$SCRIPT_DIR/target/release/icarmeter"
if [[ ! -f "$BIN_PATH" ]]; then
    echo -e "\033[0;31mKhông tìm thấy file thực thi: $BIN_PATH\033[0m"
    exit 1
fi

echo -e "${GREEN}[2/2] Khởi chạy Car Dashboard trên $FB_DEV...${NC}"
if [[ ! -w "$FB_DEV" && $EUID -ne 0 ]]; then
    echo -e "${YELLOW}-> Yêu cầu quyền ghi $FB_DEV (chạy với sudo)...${NC}"
    sudo "$BIN_PATH"
else
    "$BIN_PATH"
fi
