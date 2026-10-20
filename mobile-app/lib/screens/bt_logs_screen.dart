import 'package:flutter/material.dart';
import '../theme/app_theme.dart';
import '../services/app_state.dart';

class BtLogsScreen extends StatefulWidget {
  final AppState appState;

  const BtLogsScreen({super.key, required this.appState});

  @override
  State<BtLogsScreen> createState() => _BtLogsScreenState();
}

class _BtLogsScreenState extends State<BtLogsScreen> {
  final ScrollController _scrollController = ScrollController();
  final TextEditingController _searchCtrl = TextEditingController();
  bool _autoScroll = true;
  String _filter = '';

  @override
  void initState() {
    super.initState();
    _searchCtrl.addListener(() {
      setState(() {
        _filter = _searchCtrl.text.trim().toLowerCase();
      });
    });
  }

  @override
  void dispose() {
    _scrollController.dispose();
    _searchCtrl.dispose();
    super.dispose();
  }

  Color _getLogColor(String line) {
    if (line.contains('[BT-SPP]')) return AppColors.primary;
    if (line.contains('[BT-PAIR]')) return AppColors.warning;
    if (line.contains('[PI-LOG]')) return AppColors.success;
    if (line.contains('error') || line.contains('Error') || line.contains('failed')) {
      return AppColors.danger;
    }
    return AppColors.textSecondary;
  }

  void _requestLogs() {
    if (widget.appState.isBtConnected) {
      widget.appState.sendBtCommand({'type': 'get_bt_logs'});
    } else {
      widget.appState.wifiWs.send({'type': 'get_bt_logs'});
    }
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(
        content: Text('Đã gửi yêu cầu lấy nhật ký từ Pi...'),
        backgroundColor: AppColors.primary,
        duration: Duration(seconds: 1),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final allLogs = widget.appState.btLogs;
    final filteredLogs = _filter.isEmpty
        ? allLogs
        : allLogs.where((l) => l.toLowerCase().contains(_filter)).toList();

    if (_autoScroll && _scrollController.hasClients) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (_scrollController.hasClients) {
          _scrollController.jumpTo(_scrollController.position.maxScrollExtent);
        }
      });
    }

    return Scaffold(
      backgroundColor: Colors.transparent,
      body: Padding(
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 16),
        child: Column(
          children: [
            // Control Bar
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _searchCtrl,
                    style: const TextStyle(fontSize: 12, color: AppColors.text),
                    decoration: InputDecoration(
                      hintText: 'Lọc nhật ký (bluetoothctl, spp, pair)...',
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
                  onPressed: _requestLogs,
                  icon: const Icon(Icons.download, color: AppColors.primary, size: 20),
                  tooltip: 'Tải nhật ký mới nhất từ Pi',
                ),
                IconButton(
                  onPressed: () => widget.appState.clearLogs(),
                  icon: const Icon(Icons.delete_outline, color: AppColors.textMuted, size: 20),
                  tooltip: 'Xóa màn hình nhật ký',
                ),
                IconButton(
                  onPressed: () => setState(() => _autoScroll = !_autoScroll),
                  icon: Icon(
                    _autoScroll ? Icons.vertical_align_bottom : Icons.pause_circle_outline,
                    color: _autoScroll ? AppColors.primary : AppColors.textMuted,
                    size: 20,
                  ),
                  tooltip: _autoScroll ? 'Đang tự cuộn xuống' : 'Tạm dừng tự cuộn',
                ),
              ],
            ),
            const SizedBox(height: 10),

            // Log Console Terminal Window
            Expanded(
              child: Container(
                width: double.infinity,
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: const Color(0xFF070A10),
                  borderRadius: BorderRadius.circular(14),
                  border: Border.all(color: AppColors.border, width: 1.2),
                ),
                child: filteredLogs.isEmpty
                    ? const Center(
                        child: Text(
                          'Chưa có dữ liệu nhật ký.\nNhấn biểu tượng tải xuống để lấy log từ Raspberry Pi.',
                          style: TextStyle(fontSize: 12, color: AppColors.textMuted),
                          textAlign: TextAlign.center,
                        ),
                      )
                    : ListView.builder(
                        controller: _scrollController,
                        physics: const BouncingScrollPhysics(),
                        itemCount: filteredLogs.length,
                        itemBuilder: (context, index) {
                          final line = filteredLogs[index];
                          final color = _getLogColor(line);

                          return Padding(
                            padding: const EdgeInsets.symmetric(vertical: 2),
                            child: SelectableText(
                              line,
                              style: TextStyle(
                                fontSize: 11,
                                fontFamily: 'monospace',
                                color: color,
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
      ),
    );
  }
}
