import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../widgets/car_tpms_diagram.dart';
import '../services/native_bridge.dart';

class TpmsScreen extends StatefulWidget {
  final AppState appState;

  const TpmsScreen({super.key, required this.appState});

  @override
  State<TpmsScreen> createState() => _TpmsScreenState();
}

class _TpmsScreenState extends State<TpmsScreen> {
  late TextEditingController _flCtrl;
  late TextEditingController _frCtrl;
  late TextEditingController _rlCtrl;
  late TextEditingController _rrCtrl;
  late double _pressureAlert;
  late double _tempAlert;

  bool _isAdvertising = false;
  final double _testPressure = 2.3;
  final double _testTemp = 28.0;

  @override
  void initState() {
    super.initState();
    final cfg = widget.appState.config;
    _flCtrl = TextEditingController(text: cfg.flSensorId);
    _frCtrl = TextEditingController(text: cfg.frSensorId);
    _rlCtrl = TextEditingController(text: cfg.rlSensorId);
    _rrCtrl = TextEditingController(text: cfg.rrSensorId);
    _pressureAlert = cfg.pressureAlert;
    _tempAlert = cfg.highTempAlert;
  }

  @override
  void dispose() {
    _flCtrl.dispose();
    _frCtrl.dispose();
    _rlCtrl.dispose();
    _rrCtrl.dispose();
    if (_isAdvertising) {
      NativeBridge.stopBleAdvertising();
    }
    super.dispose();
  }

  void _saveSettings() {
    final newCfg = widget.appState.config.copyWith(
      flSensorId: _flCtrl.text.trim(),
      frSensorId: _frCtrl.text.trim(),
      rlSensorId: _rlCtrl.text.trim(),
      rrSensorId: _rrCtrl.text.trim(),
      pressureAlert: _pressureAlert,
      highTempAlert: _tempAlert,
    );

    widget.appState.saveConfig(newCfg);
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Đã lưu cấu hình cảm biến áp suất lốp!'),
        backgroundColor: AppColors.success,
      ),
    );
  }

  void _toggleBleAdvertising() async {
    if (_isAdvertising) {
      await NativeBridge.stopBleAdvertising();
      setState(() => _isAdvertising = false);
    } else {
      final phoneMac = await NativeBridge.getDeviceBluetoothAddress();
      final ok = await NativeBridge.startBleAdvertising(
        phoneMac: phoneMac.isNotEmpty ? phoneMac : '11:22:33:44:55:66',
        pressureBar: _testPressure,
        tempC: _testTemp,
      );
      if (ok) {
        setState(() => _isAdvertising = true);
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Đang phát tín hiệu BLE TPMS mô phỏng (Sysgration)...'),
              backgroundColor: AppColors.primary,
            ),
          );
        }
      } else {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Không thể kích hoạt BLE Advertising trên máy này.'),
              backgroundColor: AppColors.danger,
            ),
          );
        }
      }
    }
  }

  Widget _buildSensorInput(String label, TextEditingController ctrl) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          label,
          style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w600, color: AppColors.textSecondary),
        ),
        const SizedBox(height: 4),
        TextField(
          controller: ctrl,
          style: const TextStyle(fontSize: 13, color: AppColors.text, fontFamily: 'monospace'),
          decoration: InputDecoration(
            isDense: true,
            filled: true,
            fillColor: AppColors.surface,
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
    );
  }

  @override
  Widget build(BuildContext context) {
    final tpms = widget.appState.telemetry.tpms;
    final pressureUnit = widget.appState.storage.pressureUnit;
    final tempUnit = widget.appState.storage.tempUnit;

    return SingleChildScrollView(
      physics: const BouncingScrollPhysics(),
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // 4-Wheel Graphical Diagram
          CarTpmsDiagram(
            tpms: tpms,
            pressureAlert: _pressureAlert,
            tempAlert: _tempAlert,
            pressureUnit: pressureUnit,
            tempUnit: tempUnit,
          ),

          const SizedBox(height: 20),

          // Sensor ID Settings Card
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
                    Icon(Icons.qr_code, color: AppColors.primary, size: 20),
                    SizedBox(width: 8),
                    Text(
                      'Gán mã cảm biến (MAC Address / Hex)',
                      style: TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.bold,
                        color: AppColors.text,
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 14),
                Row(
                  children: [
                    Expanded(child: _buildSensorInput('Trước Trái (FL)', _flCtrl)),
                    const SizedBox(width: 12),
                    Expanded(child: _buildSensorInput('Trước Phải (FR)', _frCtrl)),
                  ],
                ),
                const SizedBox(height: 12),
                Row(
                  children: [
                    Expanded(child: _buildSensorInput('Sau Trái (RL)', _rlCtrl)),
                    const SizedBox(width: 12),
                    Expanded(child: _buildSensorInput('Sau Phải (RR)', _rrCtrl)),
                  ],
                ),
                const SizedBox(height: 18),

                // Pressure Threshold Slider
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Text('Cảnh báo áp suất thấp:', style: TextStyle(fontSize: 12, color: AppColors.textSecondary)),
                    Text('${_pressureAlert.toStringAsFixed(1)} bar', style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: AppColors.primary)),
                  ],
                ),
                Slider(
                  value: _pressureAlert,
                  min: 1.4,
                  max: 2.6,
                  divisions: 12,
                  activeColor: AppColors.primary,
                  inactiveColor: AppColors.border,
                  onChanged: (v) => setState(() => _pressureAlert = v),
                ),

                // Temp Threshold Slider
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Text('Cảnh báo nhiệt độ cao:', style: TextStyle(fontSize: 12, color: AppColors.textSecondary)),
                    Text('${_tempAlert.toStringAsFixed(0)}°C', style: const TextStyle(fontSize: 13, fontWeight: FontWeight.bold, color: AppColors.danger)),
                  ],
                ),
                Slider(
                  value: _tempAlert,
                  min: 50.0,
                  max: 90.0,
                  divisions: 8,
                  activeColor: AppColors.danger,
                  inactiveColor: AppColors.border,
                  onChanged: (v) => setState(() => _tempAlert = v),
                ),

                const SizedBox(height: 12),
                SizedBox(
                  width: double.infinity,
                  height: 46,
                  child: ElevatedButton(
                    onPressed: _saveSettings,
                    style: ElevatedButton.styleFrom(
                      backgroundColor: AppColors.primary,
                      foregroundColor: Colors.black,
                      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                    ),
                    child: const Text('Lưu cấu hình cảm biến', style: TextStyle(fontWeight: FontWeight.bold)),
                  ),
                ),
              ],
            ),
          ),

          const SizedBox(height: 16),

          // BLE Simulation Card
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
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    const Row(
                      children: [
                        Icon(Icons.cell_tower, color: AppColors.info, size: 20),
                        SizedBox(width: 8),
                        Text(
                          'Mô phỏng phát tín hiệu BLE TPMS',
                          style: TextStyle(fontSize: 14, fontWeight: FontWeight.bold, color: AppColors.text),
                        ),
                      ],
                    ),
                    Switch(
                      value: _isAdvertising,
                      activeThumbColor: AppColors.primary,
                      onChanged: (v) => _toggleBleAdvertising(),
                    ),
                  ],
                ),
                const SizedBox(height: 6),
                const Text(
                  'Phát gói tin BLE định dạng Sysgration (0xACAC) từ điện thoại để kiểm tra bộ thu trên xe.',
                  style: TextStyle(fontSize: 11, color: AppColors.textSecondary, height: 1.4),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
