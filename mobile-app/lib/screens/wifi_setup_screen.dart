import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../services/native_bridge.dart';

class WifiSetupScreen extends StatefulWidget {
  final AppState appState;
  final VoidCallback onNavigateToBt;

  const WifiSetupScreen({
    super.key,
    required this.appState,
    required this.onNavigateToBt,
  });

  @override
  State<WifiSetupScreen> createState() => _WifiSetupScreenState();
}

class _WifiSetupScreenState extends State<WifiSetupScreen> {
  late TextEditingController _hostCtrl;

  @override
  void initState() {
    super.initState();
    _hostCtrl = TextEditingController(text: widget.appState.wifiWs.host);
  }

  @override
  void dispose() {
    _hostCtrl.dispose();
    super.dispose();
  }

  void _bindWifi() async {
    final ok = await NativeBridge.bindToCarWifi();
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(ok
              ? 'Đã ưu tiên định tuyến gói tin qua Wi-Fi Raspberry Pi!'
              : 'Không thể định tuyến qua Wi-Fi, hãy kiểm tra kết nối.'),
          backgroundColor: ok ? AppColors.success : AppColors.danger,
        ),
      );
    }
  }

  void _reconnectWs() {
    widget.appState.wifiWs.host = _hostCtrl.text.trim();
    widget.appState.wifiWs.disconnect();
    widget.appState.wifiWs.connect();
  }

  Widget _buildStep(int number, String title, String description) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 28,
            height: 28,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: AppColors.primary.withOpacity(0.15),
              shape: BoxShape.circle,
              border: Border.all(color: AppColors.primary, width: 1.2),
            ),
            child: Text(
              number.toString(),
              style: const TextStyle(
                fontSize: 13,
                fontWeight: FontWeight.bold,
                color: AppColors.primary,
              ),
            ),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
                ),
                const SizedBox(height: 3),
                Text(
                  description,
                  style: const TextStyle(fontSize: 12, color: AppColors.textSecondary, height: 1.4),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final isWifiConnected = widget.appState.isWifiConnected;

    return SingleChildScrollView(
      physics: const BouncingScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Wi-Fi WebSocket Status Banner
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: AppColors.surface,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(
                color: isWifiConnected ? AppColors.info.withOpacity(0.5) : AppColors.border,
                width: 1.2,
              ),
            ),
            child: Row(
              children: [
                Container(
                  padding: const EdgeInsets.all(10),
                  decoration: BoxDecoration(
                    color: (isWifiConnected ? AppColors.info : AppColors.textMuted).withOpacity(0.15),
                    shape: BoxShape.circle,
                  ),
                  child: Icon(
                    isWifiConnected ? Icons.wifi : Icons.wifi_off,
                    color: isWifiConnected ? AppColors.info : AppColors.textMuted,
                    size: 24,
                  ),
                ),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        isWifiConnected ? 'Đã kết nối Wi-Fi WebSocket' : 'Chưa kết nối Wi-Fi Pi',
                        style: const TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
                      ),
                      const SizedBox(height: 2),
                      Text(
                        isWifiConnected
                            ? 'Kênh Wi-Fi sẵn sàng hỗ trợ ghép nối Bluetooth.'
                            : 'Hãy vào Cài đặt Wi-Fi điện thoại và chọn mạng của Pi.',
                        style: const TextStyle(fontSize: 11, color: AppColors.textSecondary),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),

          const SizedBox(height: 16),

          // Host input
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: AppColors.surface,
              borderRadius: BorderRadius.circular(14),
              border: Border.all(color: AppColors.border, width: 1),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Địa chỉ WebSocket của Raspberry Pi',
                  style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: AppColors.textSecondary),
                ),
                const SizedBox(height: 8),
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: _hostCtrl,
                        style: const TextStyle(fontSize: 13, color: AppColors.text, fontFamily: 'monospace'),
                        decoration: InputDecoration(
                          hintText: '192.168.4.1:8080',
                          isDense: true,
                          filled: true,
                          fillColor: AppColors.background,
                          contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(10),
                            borderSide: const BorderSide(color: AppColors.border),
                          ),
                          enabledBorder: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(10),
                            borderSide: const BorderSide(color: AppColors.border),
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(width: 10),
                    ElevatedButton(
                      onPressed: _reconnectWs,
                      style: ElevatedButton.styleFrom(
                        backgroundColor: AppColors.info,
                        foregroundColor: Colors.black,
                        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                        padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
                      ),
                      child: const Text('Thử lại', style: TextStyle(fontWeight: FontWeight.bold)),
                    ),
                  ],
                ),
                const SizedBox(height: 10),
                SizedBox(
                  width: double.infinity,
                  child: OutlinedButton.icon(
                    onPressed: _bindWifi,
                    icon: const Icon(Icons.link, size: 16, color: AppColors.info),
                    label: const Text('Buộc kết nối dữ liệu qua Wi-Fi xe (Bypass 4G/LTE)', style: TextStyle(fontSize: 12, color: AppColors.info)),
                    style: OutlinedButton.styleFrom(
                      side: const BorderSide(color: AppColors.info),
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
                    ),
                  ),
                ),
              ],
            ),
          ),

          const SizedBox(height: 20),

          // Steps Guide
          Container(
            padding: const EdgeInsets.all(18),
            decoration: BoxDecoration(
              color: AppColors.surface,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(color: AppColors.border, width: 1),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Quy trình thiết lập lần đầu',
                  style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold, color: AppColors.text),
                ),
                const SizedBox(height: 16),
                _buildStep(
                  1,
                  'Kết nối Wi-Fi phát từ Pi',
                  'Mở cài đặt Wi-Fi trên điện thoại và chọn mạng "ICarMeter-Setup" (hoặc mạng do Pi phát).',
                ),
                _buildStep(
                  2,
                  'Kích hoạt ghép nối Bluetooth',
                  'Sau khi Wi-Fi WebSocket kết nối thành công, nhấn "Ghép nối Bluetooth" để hệ thống tự động đưa Pi vào chế độ chờ kết nối.',
                ),
                _buildStep(
                  3,
                  'Sử dụng hoàn toàn qua Bluetooth',
                  'Khi Bluetooth SPP đã kết nối, bạn có thể tắt Wi-Fi. Toàn bộ thông số xe, áp suất lốp và điều khiển sẽ truyền trực tiếp qua Bluetooth.',
                ),
                const SizedBox(height: 8),
                SizedBox(
                  width: double.infinity,
                  height: 48,
                  child: ElevatedButton(
                    onPressed: widget.onNavigateToBt,
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppColors.primary,
                      foregroundColor: Colors.black,
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                    ),
                    child: const Text('Chuyển sang màn hình Bluetooth', style: TextStyle(fontWeight: FontWeight.bold)),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
