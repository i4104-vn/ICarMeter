#!/usr/bin/env bash
# ==============================================================================
# Script khởi chạy môi trường giả lập & chạy thực tế cho Car Digital Dashboard
# Kích thước màn hình: 7 inch (184mm x 70mm) - Độ phân giải: 1280 x 480
# Nền tảng: Linux trực tiếp (DRM/KMS, Framebuffer /dev/fb0) & Desktop Simulator
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Màu hiển thị terminal
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${CYAN}====================================================================${NC}"
    echo -e "${PURPLE}       CAR DIGITAL DASHBOARD - RUST EMBEDDED LINUX UI               ${NC}"
    echo -e "${CYAN}====================================================================${NC}"
    echo -e "  * Kích thước vật lý : ${GREEN}7 inch (184mm x 70mm)${NC} [Tỉ lệ ~2.63:1]"
    echo -e "  * Độ phân giải      : ${GREEN}1280 x 480 pixels${NC}"
    echo -e "  * Nền tảng mục tiêu : ${YELLOW}Debian GNU/Linux / Raspberry Pi TTY HDMI (No WM, No DE)${NC}"
    echo -e "  * Driver hiển thị   : ${YELLOW}Linux Framebuffer (/dev/fb0) | DRM/KMS (/dev/dri/card0)${NC}"
    echo -e "  * Chế độ hoạt động  : ${GREEN}Tự động hoàn toàn (Automatic Telemetry Driving)${NC}"
    echo -e "${CYAN}====================================================================${NC}"
}

MODE="sim"
BUILD_PROFILE="--release"
SNAPSHOT=""
EXTRA_ARGS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dev|--debug)
            BUILD_PROFILE=""
            shift
            ;;
        --release)
            BUILD_PROFILE="--release"
            shift
            ;;
        --sim|--simulator)
            MODE="sim"
            shift
            ;;
        --fb|--framebuffer)
            MODE="fb"
            shift
            ;;
        --drm|--kms)
            MODE="drm"
            shift
            ;;
        --cage)
            MODE="cage"
            shift
            ;;
        --snapshot)
            MODE="snapshot"
            shift
            ;;
        -h|--help)
            print_banner
            echo "Cách dùng:"
            echo "  ./run.sh                  Tự động nhận diện (TTY -> Framebuffer, Desktop -> Simulator)"
            echo "  ./run.sh --fb             Ép chạy trực tiếp trên Linux Framebuffer (/dev/fb0)"
            echo "  ./run.sh --drm            Ép chạy trực tiếp trên Linux DRM/KMS (/dev/dri/card0)"
            echo "  ./run.sh --sim            Ép chạy môi trường cửa sổ mô phỏng 1280x480"
            echo "  ./run.sh --dev            Chạy với bản build debug (nhanh hơn khi compile)"
            echo "  ./run.sh --cage           Chạy trong Cage (Wayland kiosk mô phỏng không có WM/DE)"
            echo "  ./run.sh --snapshot       Render 1 frame và lưu ra file ảnh dashboard_snapshot.png"
            exit 0
            ;;
        *)
            EXTRA_ARGS+=("$1")
            shift
            ;;
    esac
done

# Tự động chuyển sang chế độ tự động dò backend nếu đang ở trong môi trường Console TTY thuần túy
if [[ -z "$DISPLAY" && -z "$WAYLAND_DISPLAY" && "$MODE" == "sim" ]]; then
    echo -e "${YELLOW}-> Phát hiện môi trường Debian / Raspberry Pi Console TTY (không có X11/Wayland)...${NC}"
    MODE="auto"
fi

print_banner

# 1. Biên dịch dự án Rust
echo -e "${YELLOW}[1/2] Đang biên dịch dự án Rust (${BUILD_PROFILE:-debug})...${NC}"
cargo build $BUILD_PROFILE

if [[ -n "$BUILD_PROFILE" ]]; then
    BIN_PATH="$SCRIPT_DIR/target/release/car_dashboard"
else
    BIN_PATH="$SCRIPT_DIR/target/debug/car_dashboard"
fi

if [[ ! -f "$BIN_PATH" ]]; then
    echo -e "\033[0;31mKhông tìm thấy file thực thi: $BIN_PATH\033[0m"
    exit 1
fi

echo -e "${GREEN}[2/2] Khởi chạy Car Dashboard...${NC}"

# 2. Khởi chạy theo chế độ được chọn
case "$MODE" in
    auto)
        echo -e "${CYAN}-> Đang tự động dò thiết bị hiển thị DRM/KMS & Framebuffer...${NC}"
        # Kiểm tra xem có quyền truy cập framebuffer hoặc card DRM không
        NEED_SUDO=0
        if [[ -e /dev/fb0 && ! -w /dev/fb0 ]]; then
            NEED_SUDO=1
        fi
        if [[ $NEED_SUDO -eq 1 && $EUID -ne 0 ]]; then
            echo -e "${YELLOW}-> Yêu cầu quyền truy cập thiết bị đồ họa (chạy với sudo)...${NC}"
            sudo "$BIN_PATH" --backend auto "${EXTRA_ARGS[@]}"
        else
            "$BIN_PATH" --backend auto "${EXTRA_ARGS[@]}"
        fi
        ;;
    sim)
        echo -e "${CYAN}-> Đang mở cửa sổ mô phỏng kích thước 1280x480 (tương đương 184mm x 70mm)...${NC}"
        "$BIN_PATH" --backend sim "${EXTRA_ARGS[@]}"
        ;;
    cage)
        if command -v cage &> /dev/null; then
            echo -e "${CYAN}-> Đang chạy qua Cage Kiosk (Mô phỏng môi trường nhúng không WM/DE)...${NC}"
            cage -d -s -- "$BIN_PATH" --backend sim "${EXTRA_ARGS[@]}"
        else
            echo -e "${YELLOW}-> Cảnh báo: 'cage' chưa được cài đặt. Chuyển sang chế độ cửa sổ simulator thông thường...${NC}"
            "$BIN_PATH" --backend sim "${EXTRA_ARGS[@]}"
        fi
        ;;
    fb)
        echo -e "${CYAN}-> Đang chạy trực tiếp trên Linux Framebuffer (/dev/fb0)...${NC}"
        if [[ ! -e /dev/fb0 ]]; then
            echo -e "\033[0;31mLỗi: Không tìm thấy /dev/fb0. Hãy đảm bảo driver framebuffer (fbcon / simplefb / efifb / drm_kms_helper) đã được bật trong kernel.\033[0m"
            exit 1
        fi

        # Kiểm tra quyền ghi vào /dev/fb0
        if [[ ! -w /dev/fb0 && $EUID -ne 0 ]]; then
            echo -e "${YELLOW}-> Yêu cầu quyền truy cập /dev/fb0 (chạy với sudo)...${NC}"
            sudo "$BIN_PATH" --backend fb "${EXTRA_ARGS[@]}"
        else
            "$BIN_PATH" --backend fb "${EXTRA_ARGS[@]}"
        fi
        ;;
    drm)
        echo -e "${CYAN}-> Đang chạy trực tiếp trên Linux DRM/KMS (/dev/dri/card0)...${NC}"
        if [[ $EUID -ne 0 && ! -w /dev/dri/card0 ]]; then
            echo -e "${YELLOW}-> Yêu cầu quyền DRM Master (chạy với sudo)...${NC}"
            sudo "$BIN_PATH" --backend drm "${EXTRA_ARGS[@]}"
        else
            "$BIN_PATH" --backend drm "${EXTRA_ARGS[@]}"
        fi
        ;;
    snapshot)
        OUT_FILE="dashboard_snapshot.png"
        echo -e "${CYAN}-> Đang render snapshot ra file '$OUT_FILE'...${NC}"
        "$BIN_PATH" --snapshot "$OUT_FILE" "${EXTRA_ARGS[@]}"
        echo -e "${GREEN}-> Hoàn tất! File đã được lưu tại: $SCRIPT_DIR/$OUT_FILE${NC}"
        ;;
esac
