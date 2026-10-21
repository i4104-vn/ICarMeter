import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';
import '../widgets/tpms/car_tpms_diagram.dart';
import '../widgets/tpms/tpms_status_banner.dart';
import '../widgets/tpms/tpms_config_card.dart';

/// Screen displaying interactive vehicle TPMS diagnostics and Bluetooth sensor configuration.
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
  String? _selectedTire = 'fl';

  final FocusNode _flFocus = FocusNode();
  final FocusNode _frFocus = FocusNode();
  final FocusNode _rlFocus = FocusNode();
  final FocusNode _rrFocus = FocusNode();

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

    widget.appState.addListener(_onAppStateChanged);
  }

  void _onAppStateChanged() {
    if (!mounted) return;
    final cfg = widget.appState.config;
    if (!_flFocus.hasFocus && _flCtrl.text != cfg.flSensorId && cfg.flSensorId.isNotEmpty) {
      _flCtrl.text = cfg.flSensorId;
    }
    if (!_frFocus.hasFocus && _frCtrl.text != cfg.frSensorId && cfg.frSensorId.isNotEmpty) {
      _frCtrl.text = cfg.frSensorId;
    }
    if (!_rlFocus.hasFocus && _rlCtrl.text != cfg.rlSensorId && cfg.rlSensorId.isNotEmpty) {
      _rlCtrl.text = cfg.rlSensorId;
    }
    if (!_rrFocus.hasFocus && _rrCtrl.text != cfg.rrSensorId && cfg.rrSensorId.isNotEmpty) {
      _rrCtrl.text = cfg.rrSensorId;
    }
    if (_pressureAlert != cfg.pressureAlert && cfg.pressureAlert > 0.0) {
      setState(() => _pressureAlert = cfg.pressureAlert);
    }
    if (_tempAlert != cfg.highTempAlert && cfg.highTempAlert > 0.0) {
      setState(() => _tempAlert = cfg.highTempAlert);
    }
  }

  @override
  void dispose() {
    widget.appState.removeListener(_onAppStateChanged);
    _flCtrl.dispose();
    _frCtrl.dispose();
    _rlCtrl.dispose();
    _rrCtrl.dispose();
    _flFocus.dispose();
    _frFocus.dispose();
    _rlFocus.dispose();
    _rrFocus.dispose();
    super.dispose();
  }

  void _selectTire(String pos) {
    setState(() => _selectedTire = pos);
    switch (pos) {
      case 'fl':
        _flFocus.requestFocus();
        break;
      case 'fr':
        _frFocus.requestFocus();
        break;
      case 'rl':
        _rlFocus.requestFocus();
        break;
      case 'rr':
        _rrFocus.requestFocus();
        break;
    }
  }

  void _saveSettings() {
    if (!widget.appState.isBtConnected) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Chưa kết nối Bluetooth với xe! Cấu hình sẽ chỉ được lưu cục bộ.'),
          backgroundColor: AppColors.warning,
        ),
      );
    }

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
        content: Text('Đã gửi cấu hình cảm biến áp suất lốp qua Bluetooth SPP!'),
        backgroundColor: AppColors.success,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: widget.appState,
      builder: (context, _) {
        final tpms = widget.appState.telemetry.tpms;
        final isConnected = widget.appState.isBtConnected;
        final pressureUnit = widget.appState.storage.pressureUnit;
        final tempUnit = widget.appState.storage.tempUnit;

        return SingleChildScrollView(
          physics: const BouncingScrollPhysics(),
          padding: const EdgeInsets.fromLTRB(16, 12, 16, 28),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Bluetooth Telemetry Live Indicator
              TpmsStatusBanner(
                isConnected: isConnected,
                onSync: widget.appState.syncTpmsFromPi,
              ),

              // 4-Wheel Graphical Diagram
              CarTpmsDiagram(
                tpms: tpms,
                pressureAlert: _pressureAlert,
                tempAlert: _tempAlert,
                pressureUnit: pressureUnit,
                tempUnit: tempUnit,
                selectedTire: _selectedTire,
                onTireTap: _selectTire,
              ),

              const SizedBox(height: 18),

              // Sensor ID Settings Card
              TpmsConfigCard(
                flCtrl: _flCtrl,
                frCtrl: _frCtrl,
                rlCtrl: _rlCtrl,
                rrCtrl: _rrCtrl,
                flFocus: _flFocus,
                frFocus: _frFocus,
                rlFocus: _rlFocus,
                rrFocus: _rrFocus,
                selectedTire: _selectedTire,
                onSelectTire: _selectTire,
                pressureAlert: _pressureAlert,
                onPressureAlertChanged: (v) => setState(() => _pressureAlert = v),
                tempAlert: _tempAlert,
                onTempAlertChanged: (v) => setState(() => _tempAlert = v),
                onReload: widget.appState.syncTpmsFromPi,
                onSave: _saveSettings,
              ),
            ],
          ),
        );
      },
    );
  }
}
