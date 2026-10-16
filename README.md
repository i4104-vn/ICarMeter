# Car Digital Dashboard (Rust Embedded Linux)

Dự án đồng hồ táp-lô ô tô (Digital Dashboard) viết bằng Rust, chạy trực tiếp trên nền Linux mà **không cần Window Manager (WM) hay Desktop Environment (DE)** (như GNOME, KDE, XFCE, Wayfire, X11). Hệ thống xuất hình trực tiếp ra cổng HDMI của Raspberry Pi hoặc màn hình máy tính qua **Linux Framebuffer (`/dev/fb0`)** và **Linux DRM/KMS**.

Dự án hỗ trợ hoàn hảo chế độ render từ **Console TTY thuần túy trên Arch Linux**, không cần bất kỳ daemon hay công cụ hỗ trợ nào khác.

---

## 1. Thông số màn hình
* **Kích thước vật lý:** 7 inch (184 mm × 70 mm)
* **Tỉ lệ khung hình (Aspect Ratio):** ~2.63 : 1
* **Độ phân giải chuẩn:** **1280 × 480 pixels** (RGB)
* **Tốc độ khung hình mục tiêu:** 60 FPS

---

## 2. Cấu trúc thư mục dự án
```
Car/
├── Cargo.toml                  # Khai báo thư viện (tiny-skia, fontdue, minifb, memmap2, libc)
├── run.sh                      # Script thông minh chạy giả lập & console TTY
├── README.md                   # Tài liệu hướng dẫn chi tiết
├── assets/
│   └── fonts/                  # Font NotoSans nhúng trực tiếp vào binary qua include_bytes
└── src/
    ├── main.rs                 # Điểm khởi chạy, xử lý tham số CLI, điều tốc 60 FPS
    ├── config.rs               # Kích thước màn hình 1280x480, 184mm x 70mm, màu sắc
    ├── model.rs                # Mô hình trạng thái xe: vận tốc, quãng đường, thời gian lái, 4 lốp (TPMS)
    ├── render/
    │   ├── mod.rs              # Điều phối vẽ giao diện vector thuần 100%
    │   ├── font.rs             # Rasterize text subpixel sắc nét và chữ chuyển sắc (gradient)
    │   ├── car_panel.rs        # Vẽ xe vector (chassis chữ nhật bo góc, 4 bánh xe và đường dẫn TPMS)
    │   ├── speed_panel.rs      # Vẽ 2 dải vòng cung neon dạ quang gradient + số tốc độ lớn
    │   └── stats_panel.rs      # Vẽ 4 dòng thông số hành trình và đường kẻ phân cách
    └── backend/
        ├── mod.rs              # Trait DisplayBackend và tự động chọn backend
        ├── tty_control.rs      # Quản lý console TTY: KD_GRAPHICS, ẩn con trỏ, raw input
        ├── framebuffer.rs      # Render trực tiếp Framebuffer (/dev/fb0) cho Arch Linux TTY
        ├── drm_kms.rs          # Xuất hình trực tiếp Linux DRM/KMS HDMI
        └── simulator.rs        # Giả lập cửa sổ trên PC (Minifb 1280x480)
```

---

## 3. Hoạt động trên Arch Linux Console TTY (Không WM, Không DE)

Khi chuyển sang Console TTY trên Arch Linux (ví dụ nhấn `Ctrl+Alt+F2` hoặc chạy không có Desktop):

1. **Khử hoàn toàn con trỏ nhấp nháy & chữ terminal:**
   - Ứng dụng tự động gọi `ioctl(KDSETMODE, KD_GRAPHICS)` để tạm thời vô hiệu hóa `fbcon` (trình xuất văn bản của Linux kernel). Màn hình sẽ hoàn toàn sạch sẽ, không có con trỏ terminal nhấp nháy hay chữ đè lên giao diện.
   - Khi bạn thoát (`Esc`, `Q` hoặc `Ctrl+C`), ứng dụng tự động khôi phục chế độ `KD_TEXT`, bật lại con trỏ và trả lại trạng thái terminal hoàn hảo như ban đầu.

2. **Tự động căn giữa màn hình (Auto-centering / Letterboxing):**
   - Nếu màn hình TTY của bạn có độ phân giải lớn hơn 1280x480 (ví dụ 1920x1080 trên màn hình máy tính), giao diện sẽ tự động được đặt ngay ngắn ở chính giữa màn hình với nền viền đen tuyền bao quanh.
   - Nếu màn hình là đúng chuẩn 1280x480 (7 inch), giao diện sẽ khớp 1:1 toàn màn hình.

3. **Điều khiển trực tiếp từ bàn phím TTY:**
   - Đọc trực tiếp các phím mũi tên `[Lên/Xuống/Trái/Phải]`, `[Space]`, `[Esc/Q]` qua cơ chế non-blocking raw TTY mà không cần bất kỳ daemon hay thư viện ngoài nào.

4. **Cách chạy trong TTY:**
   ```bash
   # Cách 1: Dùng script tự động (tự phát hiện TTY và xin quyền nếu cần)
   ./run.sh

   # Cách 2: Ép chạy trực tiếp qua Framebuffer
   sudo ./target/release/car_dashboard --backend fb
   ```

   *(Mẹo trên Arch Linux: Để chạy không cần `sudo`, chỉ cần thêm user vào nhóm video: `sudo usermod -a -G video $USER` rồi đăng nhập lại).*

---

## 4. Cách chạy môi trường giả lập trên máy tính (PC Development)

Khi có môi trường Desktop (X11 / Wayland), `run.sh` sẽ mở cửa sổ mô phỏng:

```bash
# Chạy môi trường giả lập cửa sổ kích thước 1280x480 (184mm x 70mm):
./run.sh

# Chạy bản build debug để biên dịch nhanh hơn:
./run.sh --dev

# Render 1 frame và lưu ra file ảnh snapshot kiểm tra:
./run.sh --snapshot
```

### Phím tắt điều khiển:
* `[Mũi tên Lên]` hoặc `[W]`: Tăng tốc độ xe (Accelerate).
* `[Mũi tên Xuống]` hoặc `[S]`: Giảm tốc độ / phanh (Brake).
* `[Phím Space]`: Bật / tắt chế độ tự động chạy mô phỏng (Auto-cruise: dao động tốc độ mượt mà quanh 68 km/h).
* `[Mũi tên Trái / Phải]`: Tăng / giảm áp suất lốp xe mô phỏng (TPMS).
* `[Esc]` hoặc `[Q]`: Thoát ứng dụng.

---

## 5. Chạy trên Raspberry Pi xuất HDMI

Cấu hình timing trong file `/boot/firmware/config.txt` (hoặc `/boot/config.txt`):
```ini
hdmi_group=2
hdmi_mode=87
hdmi_timings=1280 0 48 32 80 480 0 3 5 20 0 0 0 60 0 64000000 3
hdmi_drive=2
```
Khởi động lại Pi và chạy trực tiếp binary:
```bash
./target/release/car_dashboard --backend fb
# hoặc
./target/release/car_dashboard --backend drm
```
