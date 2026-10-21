import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';

/// Card containing TPMS Sensor MAC inputs and alert thresholds configuration.
class TpmsConfigCard extends StatelessWidget {
  final TextEditingController flCtrl;
  final TextEditingController frCtrl;
  final TextEditingController rlCtrl;
  final TextEditingController rrCtrl;
  final FocusNode flFocus;
  final FocusNode frFocus;
  final FocusNode rlFocus;
  final FocusNode rrFocus;
  final String? selectedTire;
  final ValueChanged<String> onSelectTire;
  final double pressureAlert;
  final ValueChanged<double> onPressureAlertChanged;
  final double tempAlert;
  final ValueChanged<double> onTempAlertChanged;
  final VoidCallback onReload;
  final VoidCallback onSave;

  const TpmsConfigCard({
    super.key,
    required this.flCtrl,
    required this.frCtrl,
    required this.rlCtrl,
    required this.rrCtrl,
    required this.flFocus,
    required this.frFocus,
    required this.rlFocus,
    required this.rrFocus,
    required this.selectedTire,
    required this.onSelectTire,
    required this.pressureAlert,
    required this.onPressureAlertChanged,
    required this.tempAlert,
    required this.onTempAlertChanged,
    required this.onReload,
    required this.onSave,
  });

  Widget _buildSensorInput({
    required String label,
    required String posKey,
    required TextEditingController ctrl,
    required FocusNode focusNode,
  }) {
    final isSelected = selectedTire == posKey;

    return GestureDetector(
      onTap: () => onSelectTire(posKey),
      child: Container(
        padding: const EdgeInsets.all(10),
        decoration: BoxDecoration(
          color: isSelected ? AppColors.surfaceLight.withOpacity(0.6) : AppColors.surface,
          borderRadius: BorderRadius.circular(12),
          border: Border.all(
            color: isSelected ? AppColors.primary : AppColors.border,
            width: isSelected ? 1.5 : 1.0,
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  label,
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.w600,
                    color: isSelected ? AppColors.primary : AppColors.textSecondary,
                  ),
                ),
                if (isSelected)
                  const Icon(Icons.check_circle, size: 14, color: AppColors.primary),
              ],
            ),
            const SizedBox(height: 6),
            TextField(
              controller: ctrl,
              focusNode: focusNode,
              style: const TextStyle(
                fontSize: 13,
                color: AppColors.text,
                fontFamily: 'monospace',
                fontWeight: FontWeight.w500,
              ),
              decoration: InputDecoration(
                isDense: true,
                filled: true,
                fillColor: const Color(0xFF0F172A),
                hintText: 'A4:C1:38:xx:xx:xx',
                hintStyle: const TextStyle(color: AppColors.textMuted, fontSize: 11),
                contentPadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(8),
                  borderSide: BorderSide.none,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: AppColors.surface,
        borderRadius: BorderRadius.circular(18),
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
                  Icon(Icons.tune, color: AppColors.primary, size: 20),
                  SizedBox(width: 8),
                  Text(
                    'Cấu hình cảm biến & ngưỡng báo',
                    style: TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.bold,
                      color: AppColors.text,
                    ),
                  ),
                ],
              ),
              TextButton.icon(
                onPressed: onReload,
                icon: const Icon(Icons.refresh, size: 14),
                label: const Text('Đọc lại', style: TextStyle(fontSize: 12)),
                style: TextButton.styleFrom(
                  foregroundColor: AppColors.primary,
                  padding: const EdgeInsets.symmetric(horizontal: 8),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),

          // Front Row
          Row(
            children: [
              Expanded(
                child: _buildSensorInput(
                  label: 'Trước Trái (FL)',
                  posKey: 'fl',
                  ctrl: flCtrl,
                  focusNode: flFocus,
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: _buildSensorInput(
                  label: 'Trước Phải (FR)',
                  posKey: 'fr',
                  ctrl: frCtrl,
                  focusNode: frFocus,
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),

          // Rear Row
          Row(
            children: [
              Expanded(
                child: _buildSensorInput(
                  label: 'Sau Trái (RL)',
                  posKey: 'rl',
                  ctrl: rlCtrl,
                  focusNode: rlFocus,
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: _buildSensorInput(
                  label: 'Sau Phải (RR)',
                  posKey: 'rr',
                  ctrl: rrCtrl,
                  focusNode: rrFocus,
                ),
              ),
            ],
          ),
          const SizedBox(height: 18),

          // Pressure Threshold Slider
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Ngưỡng cảnh báo áp suất thấp:',
                style: TextStyle(fontSize: 12, color: AppColors.textSecondary),
              ),
              Text(
                '${pressureAlert.toStringAsFixed(1)} bar',
                style: const TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.bold,
                  color: AppColors.primary,
                ),
              ),
            ],
          ),
          Slider(
            value: pressureAlert,
            min: 1.4,
            max: 2.8,
            divisions: 14,
            activeColor: AppColors.primary,
            inactiveColor: AppColors.border,
            onChanged: onPressureAlertChanged,
          ),

          // Temp Threshold Slider
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'Ngưỡng cảnh báo nhiệt độ cao:',
                style: TextStyle(fontSize: 12, color: AppColors.textSecondary),
              ),
              Text(
                '${tempAlert.toStringAsFixed(0)}°C',
                style: const TextStyle(
                  fontSize: 13,
                  fontWeight: FontWeight.bold,
                  color: AppColors.danger,
                ),
              ),
            ],
          ),
          Slider(
            value: tempAlert,
            min: 50.0,
            max: 90.0,
            divisions: 8,
            activeColor: AppColors.danger,
            inactiveColor: AppColors.border,
            onChanged: onTempAlertChanged,
          ),

          const SizedBox(height: 12),
          SizedBox(
            width: double.infinity,
            height: 46,
            child: ElevatedButton.icon(
              onPressed: onSave,
              icon: const Icon(Icons.bluetooth_audio, size: 18),
              label: const Text(
                'Lưu cấu hình qua Bluetooth',
                style: TextStyle(fontWeight: FontWeight.bold),
              ),
              style: ElevatedButton.styleFrom(
                backgroundColor: AppColors.primary,
                foregroundColor: Colors.black,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(12),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
