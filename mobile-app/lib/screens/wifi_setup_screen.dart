import 'dart:async';
import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../services/native_bridge.dart';

class WifiSetupScreen extends StatefulWidget {
  final AppState appState;

  const WifiSetupScreen({
    super.key,
    required this.appState,
  });

  @override
  State<WifiSetupScreen> createState() => _WifiSetupScreenState();
}

class _WifiSetupScreenState extends State<WifiSetupScreen> {
  bool _checking = false;

  Future<void> _checkConnection() async {
    if (_checking) return;

    setState(() => _checking = true);
    await NativeBridge.bindToCarWifi();
    widget.appState.wifiWs.connect();

    // The WebSocket uses a four-second connection timeout. Allow it to finish
    // before showing the result of this manual check.
    await Future<void>.delayed(const Duration(seconds: 4, milliseconds: 300));
    if (!mounted) return;

    final connected = widget.appState.isWifiConnected;
    setState(() => _checking = false);
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(
        SnackBar(
          content: Text(connected
              ? 'Đã kết nối với Wi-Fi của xe.'
              : 'Chưa kết nối được. Hãy kiểm tra Wi-Fi đang chọn là ICarMeter rồi thử lại.'),
          backgroundColor: connected ? AppColors.success : AppColors.danger,
        ),
      );
  }

  Widget _buildStep(int number, String title, String description) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 20),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 30,
            height: 30,
            alignment: Alignment.center,
            decoration: BoxDecoration(
              color: AppColors.primary.withOpacity(0.14),
              shape: BoxShape.circle,
              border: Border.all(color: AppColors.primary.withOpacity(0.5)),
            ),
            child: Text(
              '$number',
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
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.bold,
                    color: AppColors.text,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  description,
                  style: const TextStyle(
                    fontSize: 12,
                    color: AppColors.textSecondary,
                    height: 1.45,
                  ),
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
    return AnimatedBuilder(
      animation: widget.appState,
      builder: (context, _) {
        final isWifiConnected = widget.appState.isWifiConnected;

        return SingleChildScrollView(
          physics: const BouncingScrollPhysics(),
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 28),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              AnimatedContainer(
                duration: const Duration(milliseconds: 250),
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: AppColors.surface,
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(
                    color: isWifiConnected
                        ? AppColors.success.withOpacity(0.55)
                        : AppColors.border,
                    width: 1.2,
                  ),
                ),
                child: Row(
                  children: [
                    AnimatedContainer(
                      duration: const Duration(milliseconds: 250),
                      padding: const EdgeInsets.all(10),
                      decoration: BoxDecoration(
                        color: (isWifiConnected
                                ? AppColors.success
                                : AppColors.textMuted)
                            .withOpacity(0.15),
                        shape: BoxShape.circle,
                      ),
                      child: Icon(
                        isWifiConnected ? Icons.wifi : Icons.wifi_off,
                        color: isWifiConnected
                            ? AppColors.success
                            : AppColors.textMuted,
                        size: 24,
                      ),
                    ),
                    const SizedBox(width: 14),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            isWifiConnected
                                ? 'Đã kết nối Wi-Fi xe'
                                : 'Chưa kết nối Wi-Fi xe',
                            style: const TextStyle(
                              fontSize: 14,
                              fontWeight: FontWeight.bold,
                              color: AppColors.text,
                            ),
                          ),
                          const SizedBox(height: 3),
                          Text(
                            isWifiConnected
                                ? 'Kết nối đang hoạt động.'
                                : 'Kết nối Wi-Fi để ghép nối và đồng bộ với xe.',
                            style: const TextStyle(
                              fontSize: 11,
                              color: AppColors.textSecondary,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 16),
              Container(
                padding: const EdgeInsets.fromLTRB(18, 20, 18, 4),
                decoration: BoxDecoration(
                  color: AppColors.surface,
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(color: AppColors.border),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'Kết nối Wi-Fi của xe',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.bold,
                        color: AppColors.text,
                      ),
                    ),
                    const SizedBox(height: 18),
                    _buildStep(
                      1,
                      'Mở cài đặt Wi-Fi',
                      'Mở danh sách mạng Wi-Fi trên điện thoại.',
                    ),
                    _buildStep(
                      2,
                      'Chọn mạng ICarMeter',
                      'Nếu điện thoại báo mạng không có Internet, hãy chọn tiếp tục kết nối với mạng này.',
                    ),
                    _buildStep(
                      3,
                      'Quay lại ứng dụng',
                      'Nhấn nút bên dưới để kiểm tra kết nối. Khi Wi-Fi đã hoạt động, bạn có thể ghép nối Bluetooth.',
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 16),
              SizedBox(
                height: 50,
                child: ElevatedButton.icon(
                  onPressed: _checking ? null : _checkConnection,
                  icon: _checking
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.black,
                          ),
                        )
                      : const Icon(Icons.wifi_find, size: 19),
                  label: Text(
                    _checking ? 'Đang kiểm tra...' : 'Kiểm tra lại kết nối',
                    style: const TextStyle(fontWeight: FontWeight.bold),
                  ),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: AppColors.primary,
                    foregroundColor: Colors.black,
                    disabledBackgroundColor: AppColors.primary.withOpacity(0.65),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(13),
                    ),
                  ),
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}
