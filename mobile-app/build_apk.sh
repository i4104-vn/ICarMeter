#!/usr/bin/env bash
# ==============================================================================
# ICarMeter Mobile App - Script Build APK Android
# Hỗ trợ biên dịch riêng biệt cho:
#   - arm64-v8a : Dành cho smartphone 64-bit hiện đại & màn hình Android ô tô
#   - armeabi-v7a: Dành cho thiết bị 32-bit cũ
#   - x86_64    : Dành cho giả lập PC / máy ảo x64 (Waydroid, Android Studio, ...)
#   - universal : Bản APK hợp nhất đa kiến trúc
# ==============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "$(realpath "${BASH_SOURCE[0]}")")" && pwd)"
cd "$SCRIPT_DIR"

# Màu hiển thị terminal
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Thiết lập đường dẫn Flutter và Android SDK
if command -v flutter >/dev/null 2>&1; then
    FLUTTER_CMD="flutter"
elif [ -x "$HOME/Documents/flutter/bin/flutter" ]; then
    export PATH="$HOME/Documents/flutter/bin:$PATH"
    FLUTTER_CMD="$HOME/Documents/flutter/bin/flutter"
else
    echo -e "${RED}Lỗi: Không tìm thấy Flutter SDK!${NC}" >&2
    exit 1
fi

if [ -z "$ANDROID_HOME" ] && [ -d "$HOME/Android/Sdk" ]; then
    export ANDROID_HOME="$HOME/Android/Sdk"
fi

if [ -d "$ANDROID_HOME/platform-tools" ]; then
    export PATH="$ANDROID_HOME/platform-tools:$PATH"
fi

TARGET="${1:-all}"
DIST_DIR="$SCRIPT_DIR/dist"
mkdir -p "$DIST_DIR"

echo -e "${CYAN}====================================================================${NC}"
echo -e "${GREEN}          ICARMETER COMPANION - ANDROID APK BUILD TOOL              ${NC}"
echo -e "${CYAN}====================================================================${NC}"
echo -e "  * Flutter SDK : $($FLUTTER_CMD --version | head -n 1)"
echo -e "  * Android SDK : ${ANDROID_HOME:-Unknown}"
echo -e "  * Chế độ build: ${YELLOW}${TARGET}${NC}"
echo -e "  * Thư mục đích: ${GREEN}${DIST_DIR}${NC}"
echo -e "${CYAN}====================================================================${NC}"

# Cập nhật dependencies
echo -e "${YELLOW}[1/3] Đang kiểm tra dependencies (flutter pub get)...${NC}"
$FLUTTER_CMD pub get

case "$TARGET" in
    v8a|arm64)
        echo -e "${YELLOW}[2/3] Đang build APK tối ưu cho arm64-v8a...${NC}"
        $FLUTTER_CMD build apk --release --split-per-abi
        cp -v build/app/outputs/flutter-apk/app-arm64-v8a-release.apk "$DIST_DIR/ICarMeter-arm64-v8a.apk"
        cp -v "$DIST_DIR/ICarMeter-arm64-v8a.apk" "$DIST_DIR/ICarMeter.apk"
        ;;

    x64|x86_64)
        echo -e "${YELLOW}[2/3] Đang build APK cho x86_64 (Giả lập / PC)...${NC}"
        $FLUTTER_CMD build apk --release --split-per-abi
        cp -v build/app/outputs/flutter-apk/app-x86_64-release.apk "$DIST_DIR/ICarMeter-x86_64.apk"
        cp -v "$DIST_DIR/ICarMeter-x86_64.apk" "$DIST_DIR/ICarMeter.apk"
        ;;

    v7a|arm32)
        echo -e "${YELLOW}[2/3] Đang build APK cho armeabi-v7a (32-bit)...${NC}"
        $FLUTTER_CMD build apk --release --split-per-abi
        cp -v build/app/outputs/flutter-apk/app-armeabi-v7a-release.apk "$DIST_DIR/ICarMeter-armeabi-v7a.apk"
        ;;

    split)
        echo -e "${YELLOW}[2/3] Đang build tách rời từng kiến trúc (split-per-abi)...${NC}"
        $FLUTTER_CMD build apk --release --split-per-abi
        cp -v build/app/outputs/flutter-apk/app-arm64-v8a-release.apk "$DIST_DIR/ICarMeter-arm64-v8a.apk"
        cp -v build/app/outputs/flutter-apk/app-armeabi-v7a-release.apk "$DIST_DIR/ICarMeter-armeabi-v7a.apk"
        cp -v build/app/outputs/flutter-apk/app-x86_64-release.apk "$DIST_DIR/ICarMeter-x86_64.apk"
        cp -v "$DIST_DIR/ICarMeter-arm64-v8a.apk" "$DIST_DIR/ICarMeter.apk"
        ;;

    universal|fat)
        echo -e "${YELLOW}[2/3] Đang build APK Universal (tất cả kiến trúc trong 1 file)...${NC}"
        $FLUTTER_CMD build apk --release
        cp -v build/app/outputs/flutter-apk/app-release.apk "$DIST_DIR/ICarMeter-universal.apk"
        cp -v "$DIST_DIR/ICarMeter-universal.apk" "$DIST_DIR/ICarMeter.apk"
        ;;

    all|*)
        echo -e "${YELLOW}[2/3] Đang build trọn bộ APK: Split ABIs + Universal...${NC}"
        $FLUTTER_CMD build apk --release --split-per-abi
        cp -v build/app/outputs/flutter-apk/app-arm64-v8a-release.apk "$DIST_DIR/ICarMeter-arm64-v8a.apk"
        cp -v build/app/outputs/flutter-apk/app-armeabi-v7a-release.apk "$DIST_DIR/ICarMeter-armeabi-v7a.apk"
        cp -v build/app/outputs/flutter-apk/app-x86_64-release.apk "$DIST_DIR/ICarMeter-x86_64.apk"
        
        $FLUTTER_CMD build apk --release
        cp -v build/app/outputs/flutter-apk/app-release.apk "$DIST_DIR/ICarMeter-universal.apk"
        cp -v "$DIST_DIR/ICarMeter-arm64-v8a.apk" "$DIST_DIR/ICarMeter.apk"
        ;;
esac

echo -e "${GREEN}[3/3] Hoàn tất! Danh sách các file APK đã tạo trong dist/:${NC}"
ls -lh "$DIST_DIR"/*.apk
echo -e "${CYAN}====================================================================${NC}"
echo -e "  * ${GREEN}ICarMeter-arm64-v8a.apk${NC} : Khuyên dùng cho hầu hết điện thoại & màn Android xe hơi"
echo -e "  * ${YELLOW}ICarMeter-x86_64.apk${NC}    : Dùng cho giả lập trên PC (Waydroid, emulator x64)"
echo -e "  * ${YELLOW}ICarMeter-universal.apk${NC} : Bản dùng chung cho mọi thiết bị"
echo -e "${CYAN}====================================================================${NC}"
