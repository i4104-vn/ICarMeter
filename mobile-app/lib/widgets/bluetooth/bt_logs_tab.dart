import 'package:flutter/material.dart';
import '../../theme/app_theme.dart';
import '../../services/app_state.dart';

/// Tab for real-time Bluetooth daemon and connection log streaming, search, and filtering.
class BtLogsTab extends StatefulWidget {
  final AppState state;
  final VoidCallback onRequestLogs;

  const BtLogsTab({
    super.key,
    required this.state,
    required this.onRequestLogs,
  });

  @override
  State<BtLogsTab> createState() => _BtLogsTabState();
}

class _BtLogsTabState extends State<BtLogsTab> {
  final TextEditingController _logSearchCtrl = TextEditingController();
  final ScrollController _logScrollCtrl = ScrollController();
  bool _autoScroll = true;
  String _logFilter = '';

  @override
  void initState() {
    super.initState();
    _logSearchCtrl.addListener(() {
      setState(() {
        _logFilter = _logSearchCtrl.text.trim().toLowerCase();
      });
    });
  }

  @override
  void dispose() {
    _logSearchCtrl.dispose();
    _logScrollCtrl.dispose();
    super.dispose();
  }

  Color _getLogColor(String line) {
    return switch (line) {
      _ when line.contains('[BT-SPP]') => AppColors.primary,
      _ when line.contains('[BT-PAIR]') => AppColors.warning,
      _ when line.contains('[HUB-LOG]') || line.contains('[BT]') => AppColors.success,
      _ when line.contains('error') || line.contains('Error') || line.contains('FAILED') => AppColors.danger,
      _ => AppColors.textSecondary,
    };
  }

  @override
  Widget build(BuildContext context) {
    final allLogs = widget.state.btLogs;
    final filteredLogs = _logFilter.isEmpty
        ? allLogs
        : allLogs.where((l) => l.toLowerCase().contains(_logFilter)).toList();

    if (_autoScroll && _logScrollCtrl.hasClients) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (_logScrollCtrl.hasClients) {
          _logScrollCtrl.jumpTo(_logScrollCtrl.position.maxScrollExtent);
        }
      });
    }

    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
      child: Column(
        children: [
          // Search & Action Bar
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _logSearchCtrl,
                  style: const TextStyle(fontSize: 12, color: AppColors.text),
                  decoration: InputDecoration(
                    hintText: 'Lọc nhật ký (bluetoothctl, pair, spp)...',
                    hintStyle: const TextStyle(fontSize: 12, color: AppColors.textMuted),
                    prefixIcon: const Icon(Icons.search, size: 16, color: AppColors.textMuted),
                    isDense: true,
                    filled: true,
                    fillColor: AppColors.surface,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 10, vertical: 8),
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
              const SizedBox(width: 8),
              IconButton(
                onPressed: widget.onRequestLogs,
                icon: const Icon(Icons.refresh, size: 20, color: AppColors.primary),
                tooltip: 'Lấy nhật ký mới nhất',
              ),
              IconButton(
                onPressed: () => widget.state.clearLogs(),
                icon: const Icon(Icons.delete_outline, size: 20, color: AppColors.danger),
                tooltip: 'Xóa nhật ký',
              ),
            ],
          ),
          const SizedBox(height: 6),

          // Options Row (Auto scroll & counter)
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                'Hiển thị: ${filteredLogs.length} / ${allLogs.length} dòng',
                style: const TextStyle(fontSize: 11, color: AppColors.textMuted),
              ),
              Row(
                children: [
                  const Text('Tự cuộn', style: TextStyle(fontSize: 11, color: AppColors.textSecondary)),
                  Checkbox(
                    value: _autoScroll,
                    activeColor: AppColors.primary,
                    checkColor: Colors.black,
                    materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                    onChanged: (val) => setState(() => _autoScroll = val ?? true),
                  ),
                ],
              ),
            ],
          ),
          const SizedBox(height: 4),

          // Log terminal list view
          Expanded(
            child: Container(
              padding: const EdgeInsets.all(10),
              decoration: BoxDecoration(
                color: const Color(0xFF090D16),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: AppColors.border),
              ),
              child: filteredLogs.isEmpty
                  ? const Center(
                      child: Text(
                        'Chưa có dữ liệu nhật ký Bluetooth.',
                        style: TextStyle(color: AppColors.textMuted, fontSize: 12),
                      ),
                    )
                  : ListView.builder(
                      controller: _logScrollCtrl,
                      itemCount: filteredLogs.length,
                      itemBuilder: (context, i) {
                        final line = filteredLogs[i];
                        return Padding(
                          padding: const EdgeInsets.symmetric(vertical: 2.0),
                          child: SelectableText(
                            line,
                            style: TextStyle(
                              fontFamily: 'monospace',
                              fontSize: 11,
                              color: _getLogColor(line),
                              height: 1.35,
                            ),
                          ),
                        );
                      },
                    ),
            ),
          ),
        ],
      ),
    );
  }
}
