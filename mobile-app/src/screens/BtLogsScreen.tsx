import React, { useEffect, useRef, useState, useMemo } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TextInput,
  TouchableOpacity,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { colors } from '../theme/colors';
import { useApp } from '../context/AppContext';

interface BtLogsScreenProps {
  onBack?: () => void;
}

type FilterTag = 'all' | 'bluetoothctl' | 'agent' | 'rfcomm' | 'err';

export const BtLogsScreen: React.FC<BtLogsScreenProps> = ({ onBack }) => {
  const { isBtConnected, isWifiConnected, btTransport, wifiTransport, showToast } = useApp();
  const [logs, setLogs] = useState<string[]>([]);
  const [filter, setFilter] = useState<FilterTag>('all');
  const [search, setSearch] = useState('');
  const [autoScroll, setAutoScroll] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const listRef = useRef<FlatList>(null);

  const activeTransport = isBtConnected ? btTransport : isWifiConnected ? wifiTransport : null;

  const fetchLogs = () => {
    if (!activeTransport) {
      showToast('Chưa kết nối tới Pi');
      return;
    }
    setIsLoading(true);
    activeTransport.send({ type: 'get_bt_logs' });
    setTimeout(() => setIsLoading(false), 800);
  };

  const handleClear = () => {
    Alert.alert('Xác nhận', 'Xóa toàn bộ nhật ký Bluetooth trên Pi?', [
      { text: 'Hủy', style: 'cancel' },
      {
        text: 'Xóa',
        style: 'destructive',
        onPress: () => {
          if (activeTransport) {
            activeTransport.send({ type: 'clear_bt_logs' });
          }
          setLogs([]);
          showToast('Đã xóa logs');
        },
      },
    ]);
  };

  useEffect(() => {
    const handleMsg = (msg: any) => {
      if (msg.type === 'bt_logs' && Array.isArray(msg.logs)) {
        setLogs(msg.logs);
        setIsLoading(false);
      } else if (msg.type === 'bt_log_entry' && typeof msg.line === 'string') {
        setLogs((prev) => [...prev, msg.line]);
      }
    };

    const unsubBt = btTransport.onMessage(handleMsg);
    const unsubWifi = wifiTransport.onMessage(handleMsg);

    fetchLogs();

    return () => {
      unsubBt();
      unsubWifi();
    };
  }, [isBtConnected, isWifiConnected]);

  const filteredLogs = useMemo(() => {
    return logs.filter((line) => {
      // 1. Tag filter
      if (filter === 'bluetoothctl' && !line.includes('[bluetoothctl]')) return false;
      if (filter === 'agent' && !line.includes('[BTAgent]')) return false;
      if (filter === 'rfcomm' && !line.includes('[BT-RFCOMM]')) return false;
      if (
        filter === 'err' &&
        !line.includes('[ERR]') &&
        !line.toLowerCase().includes('fail') &&
        !line.toLowerCase().includes('error')
      ) {
        return false;
      }
      // 2. Search query filter
      if (search.trim()) {
        const q = search.trim().toLowerCase();
        if (!line.toLowerCase().includes(q)) return false;
      }
      return true;
    });
  }, [logs, filter, search]);

  useEffect(() => {
    if (autoScroll && filteredLogs.length > 0) {
      setTimeout(() => {
        listRef.current?.scrollToEnd({ animated: true });
      }, 80);
    }
  }, [filteredLogs.length, autoScroll]);

  const getTagColor = (line: string) => {
    if (line.includes('[ERR]') || line.includes('error') || line.includes('FAILED')) {
      return '#f87171'; // red
    }
    if (line.includes('[bluetoothctl]')) {
      return '#38bdf8'; // cyan
    }
    if (line.includes('[BTAgent]')) {
      return '#fbbf24'; // yellow
    }
    if (line.includes('[BT-RFCOMM]')) {
      return '#34d399'; // emerald
    }
    if (line.includes('[bt-exec]')) {
      return '#a78bfa'; // violet
    }
    return colors.textSecondary;
  };

  const renderLogItem = ({ item }: { item: string }) => {
    const tagColor = getTagColor(item);
    return (
      <View style={styles.logRow}>
        <Text style={[styles.logText, { color: tagColor }]}>{item}</Text>
      </View>
    );
  };

  return (
    <View style={styles.container}>
      {/* Top Header */}
      <View style={styles.header}>
        {onBack && (
          <TouchableOpacity onPress={onBack} style={styles.backBtn}>
            <Text style={styles.backBtnText}>←</Text>
          </TouchableOpacity>
        )}
        <View style={styles.headerTitles}>
          <Text style={styles.title}>Nhật Ký Bluetooth</Text>
          <View style={styles.subRow}>
            <View
              style={[
                styles.transportDot,
                isBtConnected
                  ? styles.dotBt
                  : isWifiConnected
                  ? styles.dotWifi
                  : styles.dotOffline,
              ]}
            />
            <Text style={styles.subtitle}>
              {isBtConnected
                ? 'Bluetooth SPP'
                : isWifiConnected
                ? 'Wi-Fi WS'
                : 'Chưa kết nối'}{' '}
              • {filteredLogs.length} dòng
            </Text>
          </View>
        </View>

        {/* Action Controls */}
        <View style={styles.headerActions}>
          <TouchableOpacity
            style={[styles.actionBtn, autoScroll && styles.actionBtnActive]}
            onPress={() => setAutoScroll(!autoScroll)}
          >
            <Text style={[styles.actionBtnText, autoScroll && styles.actionBtnTextActive]}>
              {autoScroll ? 'Cuộn: Bật' : 'Cuộn: Tắt'}
            </Text>
          </TouchableOpacity>

          <TouchableOpacity style={styles.actionBtn} onPress={fetchLogs}>
            {isLoading ? (
              <ActivityIndicator size="small" color={colors.accentCyan} />
            ) : (
              <Text style={styles.actionBtnText}>Tải lại</Text>
            )}
          </TouchableOpacity>

          <TouchableOpacity
            style={[styles.actionBtn, styles.actionBtnDanger]}
            onPress={handleClear}
          >
            <Text style={styles.actionBtnTextDanger}>Xóa</Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* Filter Tabs */}
      <View style={styles.filterRow}>
        {(
          [
            { id: 'all', label: 'Tất cả' },
            { id: 'bluetoothctl', label: 'bluetoothctl' },
            { id: 'agent', label: 'Agent' },
            { id: 'rfcomm', label: 'RFCOMM' },
            { id: 'err', label: 'Lỗi' },
          ] as { id: FilterTag; label: string }[]
        ).map((tab) => {
          const active = filter === tab.id;
          return (
            <TouchableOpacity
              key={tab.id}
              style={[styles.filterChip, active && styles.filterChipActive]}
              onPress={() => setFilter(tab.id)}
            >
              <Text style={[styles.filterChipText, active && styles.filterChipTextActive]}>
                {tab.label}
              </Text>
            </TouchableOpacity>
          );
        })}
      </View>

      {/* Search Input */}
      <View style={styles.searchRow}>
        <TextInput
          style={styles.searchInput}
          placeholder="Tìm kiếm log..."
          placeholderTextColor={colors.textMuted}
          value={search}
          onChangeText={setSearch}
          autoCapitalize="none"
          autoCorrect={false}
        />
        {search.length > 0 && (
          <TouchableOpacity onPress={() => setSearch('')} style={styles.clearSearchBtn}>
            <Text style={styles.clearSearchText}>✕</Text>
          </TouchableOpacity>
        )}
      </View>

      {/* Terminal Log Console */}
      <View style={styles.consoleContainer}>
        <FlatList
          ref={listRef}
          data={filteredLogs}
          keyExtractor={(_, index) => String(index)}
          renderItem={renderLogItem}
          ListEmptyComponent={
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyText}>
                {isLoading ? 'Đang tải logs...' : 'Chưa có dữ liệu logs'}
              </Text>
            </View>
          }
          contentContainerStyle={styles.listContent}
          initialNumToRender={40}
          maxToRenderPerBatch={30}
        />
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.bgPrimary,
    paddingHorizontal: 14,
    paddingTop: 12,
    paddingBottom: 72,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 10,
  },
  backBtn: {
    padding: 6,
    marginRight: 6,
  },
  backBtnText: {
    color: colors.textPrimary,
    fontSize: 20,
    fontWeight: '700',
  },
  headerTitles: {
    flex: 1,
  },
  title: {
    fontSize: 16,
    fontWeight: '800',
    color: colors.textPrimary,
  },
  subRow: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    marginTop: 2,
  },
  transportDot: {
    width: 7,
    height: 7,
    borderRadius: 4,
  },
  dotBt: {
    backgroundColor: '#34d399',
  },
  dotWifi: {
    backgroundColor: '#38bdf8',
  },
  dotOffline: {
    backgroundColor: '#f87171',
  },
  subtitle: {
    fontSize: 11,
    color: colors.textSecondary,
  },
  headerActions: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
  },
  actionBtn: {
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
    paddingHorizontal: 8,
    paddingVertical: 5,
    borderRadius: 6,
  },
  actionBtnActive: {
    borderColor: colors.accentCyan,
    backgroundColor: 'rgba(56, 189, 248, 0.15)',
  },
  actionBtnText: {
    color: colors.textPrimary,
    fontSize: 11,
    fontWeight: '600',
  },
  actionBtnTextActive: {
    color: colors.accentCyan,
    fontWeight: '700',
  },
  actionBtnDanger: {
    borderColor: 'rgba(239, 68, 68, 0.4)',
  },
  actionBtnTextDanger: {
    color: '#f87171',
    fontSize: 11,
    fontWeight: '600',
  },
  filterRow: {
    flexDirection: 'row',
    gap: 6,
    marginBottom: 8,
    flexWrap: 'wrap',
  },
  filterChip: {
    paddingHorizontal: 9,
    paddingVertical: 4,
    borderRadius: 14,
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
  },
  filterChipActive: {
    backgroundColor: 'rgba(56, 189, 248, 0.2)',
    borderColor: colors.accentCyan,
  },
  filterChipText: {
    fontSize: 11,
    color: colors.textSecondary,
    fontWeight: '600',
  },
  filterChipTextActive: {
    color: colors.accentCyan,
    fontWeight: '700',
  },
  searchRow: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 8,
    paddingHorizontal: 10,
    marginBottom: 10,
  },
  searchInput: {
    flex: 1,
    paddingVertical: 6,
    color: colors.textPrimary,
    fontSize: 12,
  },
  clearSearchBtn: {
    padding: 4,
  },
  clearSearchText: {
    color: colors.textMuted,
    fontSize: 12,
  },
  consoleContainer: {
    flex: 1,
    backgroundColor: '#0a0f1d',
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#1e293b',
    overflow: 'hidden',
  },
  listContent: {
    padding: 8,
  },
  logRow: {
    paddingVertical: 2,
  },
  logText: {
    fontFamily: 'monospace',
    fontSize: 10.5,
    lineHeight: 16,
  },
  emptyContainer: {
    paddingVertical: 50,
    alignItems: 'center',
  },
  emptyText: {
    color: colors.textMuted,
    fontSize: 12,
  },
});
