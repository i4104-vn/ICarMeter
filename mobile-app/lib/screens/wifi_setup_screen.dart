import 'dart:async';
import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../services/platform/native_bridge.dart';

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
    await widget.appState.wifiWs.connect();
    await widget.appState.matchWithPi();

    // The WebSocket uses a four-second connection timeout. Allow it to finish
    // before showing the result of this manual check.
    await Future<void>.delayed(const Duration(seconds: 4, milliseconds: 300));
    if (!mounted) return;

    final connected = widget.appState.isWifiConnected;
    if (connected) {
      await widget.appState.matchWithPi();
    }
    setState(() => _checking = false);
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(
        SnackBar(
          content: Text(connected
              ? 'Đã kết nối với Wi-Fi của xe và đang khớp nối dữ liệu.'
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

  Widget _buildMatchingCard(AppState state) {
    final isBtConnected = state.isBtConnected;
    final isBtConnecting = state.isBtConnecting;
    final piMac = state.btCoordinator.piMac;
    final piName = state.btCoordinator.piName;

    return Container(
      margin: const EdgeInsets.only(top: 16),
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: AppColors.surface,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(
          color: isBtConnected
              ? AppColors.success.withOpacity(0.6)
              : AppColors.primary.withOpacity(0.5),
          width: 1.2,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: (isBtConnected ? AppColors.success : AppColors.primary)
                      .withOpacity(0.18),
                  shape: BoxShape.circle,
                ),
                child: Icon(
                  isBtConnected ? Icons.bluetooth_connected : Icons.sync_alt,
                  color: isBtConnected ? AppColors.success : AppColors.primary,
                  size: 22,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      isBtConnected
                          ? 'Đã khớp nối Bluetooth với xe'
                          : 'Khớp nối xe (ICarMeter)',
                      style: const TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.bold,
                        color: AppColors.text,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      isBtConnected
                          ? 'Đang đồng bộ dữ liệu qua Bluetooth SPP.'
                          : (piMac.isNotEmpty
                              ? 'Đã nhận diện: $piName ($piMac)'
                              : 'Đã kết nối Wi-Fi, sẵn sàng ghép nối với xe.'),
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
          const SizedBox(height: 14),
          if (isBtConnected)
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              decoration: BoxDecoration(
                color: AppColors.success.withOpacity(0.1),
                borderRadius: BorderRadius.circular(10),
              ),
              child: Row(
                children: [
                  const Icon(Icons.check_circle, color: AppColors.success, size: 16),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Kết nối hoàn tất với $piName ($piMac)',
                      style: const TextStyle(fontSize: 12, color: AppColors.success),
                    ),
                  ),
                ],
              ),
            )
          else
            SizedBox(
              height: 44,
              child: ElevatedButton.icon(
                onPressed: isBtConnecting
                    ? null
                    : () {
                        if (piMac.isNotEmpty) {
                          state.connectToPi(piMac);
                        } else {
                          state.matchWithPi();
                        }
                      },
                icon: isBtConnecting
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          color: Colors.black,
                        ),
                      )
                    : const Icon(Icons.link, size: 18),
                label: Text(
                  isBtConnecting
                      ? 'Đang kết nối Bluetooth...'
                      : (piMac.isNotEmpty
                          ? 'Ghép nối & Kết nối Bluetooth ngay'
                          : 'Nhận diện & Khớp nối với xe'),
                  style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 13),
                ),
                style: ElevatedButton.styleFrom(
                  backgroundColor: AppColors.secondary,
                  foregroundColor: Colors.black,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(11),
                  ),
                ),
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
                                ? 'Kết nối Wi-Fi (192.168.4.1) đang hoạt động.'
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
              if (isWifiConnected) _buildMatchingCard(widget.appState),
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
                      'Nhấn nút bên dưới để kiểm tra kết nối. Khi Wi-Fi đã hoạt động, ứng dụng sẽ tự động nhận diện và ghép nối với xe.',
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
