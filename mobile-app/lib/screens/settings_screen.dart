import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../services/native_bridge.dart';

class SettingsScreen extends StatefulWidget {
  final AppState appState;

  const SettingsScreen({super.key, required this.appState});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
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

  void _saveUnits() async {
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
          content: Text('Đã cập nhật cài đặt ứng dụng!'),
          backgroundColor: AppColors.success,
        ),
      );
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
    return SingleChildScrollView(
      physics: const BouncingScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Units Settings Card
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
                  options: {'kmh': 'km/h', 'mph': 'mph'},
                  onSelected: (v) => _speedUnit = v,
                ),
                const Divider(color: AppColors.border, height: 16),
                _buildSegmentedChoice<String>(
                  title: 'Đơn vị áp suất lốp',
                  selected: _pressureUnit,
                  options: {'bar': 'bar', 'psi': 'psi'},
                  onSelected: (v) => _pressureUnit = v,
                ),
                const Divider(color: AppColors.border, height: 16),
                _buildSegmentedChoice<String>(
                  title: 'Đơn vị nhiệt độ',
                  selected: _tempUnit,
                  options: {'C': '°C', 'F': '°F'},
                  onSelected: (v) => _tempUnit = v,
                ),
              ],
            ),
          ),

          const SizedBox(height: 16),

          // Connectivity & System Settings
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
                    Icon(Icons.settings_suggest, color: AppColors.primary, size: 20),
                    SizedBox(width: 8),
                    Text(
                      'Kết nối & Dịch vụ nền',
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
                          Text('Luôn giữ kênh chờ và tự kết nối khi phát hiện thiết bị', style: TextStyle(fontSize: 11, color: AppColors.textSecondary)),
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
                          Text('Dịch vụ GPS', style: TextStyle(fontSize: 13, color: AppColors.text)),
                          SizedBox(height: 2),
                          Text('Duy trì định vị GPS liên tục khi ứng dụng chạy ẩn', style: TextStyle(fontSize: 11, color: AppColors.textSecondary)),
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
                  'Địa chỉ MAC mặc định của thiết bị',
                  style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: AppColors.textSecondary),
                ),
                const SizedBox(height: 6),
                TextField(
                  controller: _piMacCtrl,
                  style: const TextStyle(fontSize: 13, color: AppColors.text, fontFamily: 'monospace'),
                  decoration: InputDecoration(
                    hintText: 'AA:BB:CC:DD:EE:FF',
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
              ],
            ),
          ),

          const SizedBox(height: 16),

          SizedBox(
            width: double.infinity,
            height: 48,
            child: ElevatedButton(
              onPressed: _saveUnits,
              style: ElevatedButton.styleFrom(
                backgroundColor: AppColors.primary,
                foregroundColor: Colors.black,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
              ),
              child: const Text('Lưu cài đặt', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 15)),
            ),
          ),

          const SizedBox(height: 24),

          // App Info
          Container(
            padding: const EdgeInsets.all(16),
            alignment: Alignment.center,
            child: Column(
              children: const [
                Text(
                  'ICarMeter Companion',
                  style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.textSecondary),
                ),
                SizedBox(height: 4),
                Text(
                  'Phiên bản: 1.0.0',
                  style: TextStyle(fontSize: 11, color: AppColors.textMuted),
                ),
                SizedBox(height: 2),
                Text(
                  'Bluetooth SPP',
                  style: TextStyle(fontSize: 10, color: AppColors.textMuted),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
