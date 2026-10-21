import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';
import '../../services/app_state.dart';
import '../../services/platform/native_bridge.dart';

/// Form widget containing unit preferences, connection switches, and Pi MAC address.
class SettingsForm extends StatefulWidget {
  final AppState appState;
  final VoidCallback? onSaved;
  final ScrollController? scrollController;

  const SettingsForm({
    super.key,
    required this.appState,
    this.onSaved,
    this.scrollController,
  });

  @override
  State<SettingsForm> createState() => _SettingsFormState();
}

class _SettingsFormState extends State<SettingsForm> {
  late String _speedUnit;
  late String _pressureUnit;
  late String _tempUnit;
  late bool _autoReconnect;
  late bool _gpsForeground;
  late TextEditingController _piMacCtrl;

  @override
  void initState() {
    super.initState();
    final s = widget.appState.storage;
    _speedUnit = s.speedUnit;
    _pressureUnit = s.pressureUnit;
    _tempUnit = s.tempUnit;
    _autoReconnect = s.autoReconnect;
    _gpsForeground = s.gpsForeground;
    _piMacCtrl = TextEditingController(text: s.piMac);
  }

  @override
  void dispose() {
    _piMacCtrl.dispose();
    super.dispose();
  }

  void _saveSettings() async {
    final s = widget.appState.storage;
    await s.setSpeedUnit(_speedUnit);
    await s.setPressureUnit(_pressureUnit);
    await s.setTempUnit(_tempUnit);
    await s.setAutoReconnect(_autoReconnect);
    await s.setGpsForeground(_gpsForeground);
    await s.setPiMac(_piMacCtrl.text.trim());

    if (_gpsForeground) {
      await NativeBridge.startForegroundService();
      await NativeBridge.startGps(rateHz: 2);
    } else {
      await NativeBridge.stopForegroundService();
      await NativeBridge.stopGps();
    }

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Đã lưu cấu hình cài đặt thành công!'),
          backgroundColor: AppColors.success,
          duration: Duration(seconds: 2),
        ),
      );
      widget.onSaved?.call();
    }
  }

  Widget _buildSegmentedChoice<T>({
    required String title,
    required T selected,
    required Map<T, String> options,
    required ValueChanged<T> onSelected,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(title, style: const TextStyle(fontSize: 13, color: AppColors.text)),
          Container(
            decoration: BoxDecoration(
              color: AppColors.background,
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: AppColors.border),
            ),
            child: Row(
              children: options.entries.map((e) {
                final isCur = e.key == selected;
                return GestureDetector(
                  onTap: () {
                    onSelected(e.key);
                    setState(() {});
                  },
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 6),
                    decoration: BoxDecoration(
                      color: isCur ? AppColors.primary : Colors.transparent,
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Text(
                      e.value,
                      style: TextStyle(
                        fontSize: 12,
                        fontWeight: isCur ? FontWeight.bold : FontWeight.normal,
                        color: isCur ? Colors.black : AppColors.textSecondary,
                      ),
                    ),
                  ),
                );
              }).toList(),
            ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListView(
      controller: widget.scrollController,
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 32),
      children: [
        // Units Card
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: AppColors.surface,
            borderRadius: BorderRadius.circular(16),
            border: Border.all(color: AppColors.border, width: 1),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Row(
                children: [
                  Icon(Icons.tune, color: AppColors.primary, size: 20),
                  SizedBox(width: 8),
                  Text(
                    'Đơn vị đo lường',
                    style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              _buildSegmentedChoice<String>(
                title: 'Đơn vị tốc độ',
                selected: _speedUnit,
                options: const {'kmh': 'KM/H', 'mph': 'MPH'},
                onSelected: (v) => _speedUnit = v,
              ),
              const Divider(color: AppColors.border, height: 12),
              _buildSegmentedChoice<String>(
                title: 'Đơn vị áp suất',
                selected: _pressureUnit,
                options: const {'bar': 'Bar', 'psi': 'PSI', 'kpa': 'kPa'},
                onSelected: (v) => _pressureUnit = v,
              ),
              const Divider(color: AppColors.border, height: 12),
              _buildSegmentedChoice<String>(
                title: 'Đơn vị nhiệt độ',
                selected: _tempUnit,
                options: const {'C': '°C', 'F': '°F'},
                onSelected: (v) => _tempUnit = v,
              ),
            ],
          ),
        ),

        const SizedBox(height: 16),

        // Connection Preferences
        Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: AppColors.surface,
            borderRadius: BorderRadius.circular(16),
            border: Border.all(color: AppColors.border, width: 1),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Row(
                children: [
                  Icon(Icons.sync_alt, color: AppColors.secondary, size: 20),
                  SizedBox(width: 8),
                  Text(
                    'Tùy chọn kết nối',
                    style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              Row(
                children: [
                  const Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Tự động kết nối lại Bluetooth', style: TextStyle(fontSize: 13, color: AppColors.text)),
                        SizedBox(height: 2),
                        Text('Tự kết nối với thiết bị khi bật app', style: TextStyle(fontSize: 11, color: AppColors.textSecondary)),
                      ],
                    ),
                  ),
                  Switch(
                    value: _autoReconnect,
                    activeThumbColor: AppColors.primary,
                    onChanged: (v) => setState(() => _autoReconnect = v),
                  ),
                ],
              ),
              const Divider(color: AppColors.border, height: 16),
              Row(
                children: [
                  const Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Dịch vụ GPS chạy nền', style: TextStyle(fontSize: 13, color: AppColors.text)),
                        SizedBox(height: 2),
                        Text('Duy trì định vị GPS khi ẩn ứng dụng', style: TextStyle(fontSize: 11, color: AppColors.textSecondary)),
                      ],
                    ),
                  ),
                  Switch(
                    value: _gpsForeground,
                    activeThumbColor: AppColors.primary,
                    onChanged: (v) => setState(() => _gpsForeground = v),
                  ),
                ],
              ),
              const Divider(color: AppColors.border, height: 16),
              const SizedBox(height: 4),
              const Text(
                'Địa chỉ MAC lưu mặc định',
                style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: AppColors.textSecondary),
              ),
              const SizedBox(height: 6),
              TextField(
                controller: _piMacCtrl,
                style: const TextStyle(fontSize: 13, color: AppColors.text, fontFamily: 'monospace'),
                decoration: InputDecoration(
                  hintText: 'AA:BB:CC:DD:EE:FF',
                  hintStyle: const TextStyle(color: AppColors.textMuted),
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
                  focusedBorder: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(10),
                    borderSide: const BorderSide(color: AppColors.primary),
                  ),
                ),
              ),
            ],
          ),
        ),

        const SizedBox(height: 24),

        // Save Button
        ElevatedButton.icon(
          onPressed: _saveSettings,
          icon: const Icon(Icons.check, size: 20),
          label: const Text(
            'Lưu cài đặt',
            style: TextStyle(fontSize: 15, fontWeight: FontWeight.bold),
          ),
          style: ElevatedButton.styleFrom(
            backgroundColor: AppColors.primary,
            foregroundColor: Colors.black,
            padding: const EdgeInsets.symmetric(vertical: 14),
            shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
            elevation: 0,
          ),
        ),
      ],
    );
  }
}
