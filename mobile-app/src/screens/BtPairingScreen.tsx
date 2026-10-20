import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TextInput,
  ActivityIndicator,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { colors } from '../theme/colors';
import { Card } from '../components/common/Card';
import { PrimaryButton } from '../components/common/PrimaryButton';
import { BtDeviceItem } from '../components/bluetooth/BtDeviceItem';
import { useApp } from '../context/AppContext';
import { BtDevice, BtStatusData } from '../types/bluetooth';
import { NativeICarMeter } from '../native/ICarMeterNative';
import { BtLogsScreen } from './BtLogsScreen';

interface BtPairingScreenProps {
  onPairSuccess: (piMac: string) => void;
  onBackToWifi: () => void;
}

export const BtPairingScreen: React.FC<BtPairingScreenProps> = ({
  onPairSuccess,
  onBackToWifi,
}) => {
  const { wifiTransport, settings, setPhoneMac, setPiBtMac, showToast } = useApp();
  const [devices, setDevices] = useState<BtDevice[]>([]);
  const [pairedDevices, setPairedDevices] = useState<BtDevice[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [pairingMac, setPairingMac] = useState<string | null>(null);
  const [pinCode, setPinCode] = useState<string | null>(null);
  const [customMac, setCustomMac] = useState(settings.phoneMac || '');
  const [showLogs, setShowLogs] = useState(false);

  useEffect(() => {
    // 1. Get initial BT status from Pi via WiFi WS
    wifiTransport.send({ type: 'get_bt_status' });

    // 2. Start discovery on Pi
    wifiTransport.send({ type: 'bt_start_discovery' });
    setIsScanning(true);

    // 3. Listen to messages from WiFi WS
    const unsub = wifiTransport.onMessage((msg) => {
      if (msg.type === 'bt_status' && msg.data) {
        const data = msg.data as BtStatusData;
        if (data.paired_devices) {
          setPairedDevices(data.paired_devices);
        }
      } else if (msg.type === 'bt_device_found') {
        const mac = msg.mac || '';
        const name = msg.name || mac;
        const rssi = msg.rssi;
        if (mac) {
          setDevices((prev) => {
            if (prev.some((d) => d.mac.toUpperCase() === mac.toUpperCase())) {
              return prev;
            }
            return [...prev, { mac, name, rssi }];
          });
        }
      } else if (msg.type === 'bt_pin_code') {
        setPinCode(msg.pin ? msg.pin : null);
      } else if (msg.type === 'bt_pair_result') {
        setPairingMac(null);
        setPinCode(null);
        if (msg.success) {
          showToast('Ghép đôi thành công');
          finishPairing();
        } else {
          Alert.alert('Lỗi ghép đôi', msg.message || 'Không thể ghép đôi. Thử lại.');
        }
      }
    });

    return () => {
      wifiTransport.send({ type: 'bt_stop_discovery' });
      unsub();
    };
  }, []);

  const finishPairing = async () => {
    // Check phone's paired devices to find Pi's MAC
    try {
      const phonePaired = await NativeICarMeter.getPhonePairedDevices();
      const piDevice = phonePaired.find(
        (d) => d.name.toLowerCase().includes('icarmeter') || d.name.toLowerCase().includes('pi')
      );
      const piMac = piDevice ? piDevice.mac : '';
      if (piMac) {
        await setPiBtMac(piMac);
      }
      // Stop WiFi transport now that pairing is done
      wifiTransport.stop();
      onPairSuccess(piMac);
    } catch {
      wifiTransport.stop();
      onPairSuccess('');
    }
  };

  const handlePair = (targetMac: string) => {
    const cleanMac = targetMac.trim().toUpperCase();
    if (!cleanMac) {
      showToast('Chưa nhập địa chỉ MAC');
      return;
    }
    setPinCode(null);
    setPairingMac(cleanMac);
    wifiTransport.send({ type: 'bt_pair', mac: cleanMac });
  };

  const handleManualPair = () => {
    handlePair(customMac);
  };

  const isAlreadyPaired = (mac: string) => {
    return pairedDevices.some((d) => d.mac.toUpperCase() === mac.toUpperCase());
  };

  if (showLogs) {
    return <BtLogsScreen onBack={() => setShowLogs(false)} />;
  }

  return (
    <View style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <TouchableOpacity onPress={onBackToWifi} style={styles.backBtn}>
          <Text style={styles.backBtnText}>←</Text>
        </TouchableOpacity>
        <View style={styles.headerTitles}>
          <Text style={styles.title}>Ghép Đôi Bluetooth</Text>
          <Text style={styles.subtitle}>Chọn điện thoại để Pi kết nối</Text>
        </View>
        <View style={styles.headerRight}>
          <TouchableOpacity style={styles.logsBtn} onPress={() => setShowLogs(true)}>
            <Text style={styles.logsBtnText}>Logs</Text>
          </TouchableOpacity>
          {isScanning && <ActivityIndicator size="small" color={colors.accentCyan} />}
        </View>
      </View>

      {/* PIN / Passkey Display */}
      {pinCode && (
        <Card style={styles.pinCard}>
          <Text style={styles.pinLabel}>MÃ GHÉP NỐI</Text>
          <Text style={styles.pinValue}>{pinCode}</Text>
          <Text style={styles.pinSub}>Nhập hoặc xác nhận mã này trên điện thoại</Text>
        </Card>
      )}

      {/* Auto-detected Phone MAC section */}
      {settings.phoneMac ? (
        <Card style={styles.myDeviceCard}>
          <View style={styles.myDeviceRow}>
            <View>
              <Text style={styles.myDeviceLabel}>Điện thoại này</Text>
              <Text style={styles.myDeviceMac}>{settings.phoneMac}</Text>
            </View>
            <PrimaryButton
              title={pairingMac === settings.phoneMac ? 'Đang ghép...' : 'Ghép ngay'}
              onPress={() => handlePair(settings.phoneMac)}
              disabled={pairingMac !== null}
              style={styles.myDeviceBtn}
            />
          </View>
        </Card>
      ) : (
        <Card style={styles.manualCard}>
          <Text style={styles.manualLabel}>Nhập MAC điện thoại thủ công</Text>
          <View style={styles.manualRow}>
            <TextInput
              style={styles.input}
              placeholder="AA:BB:CC:DD:EE:FF"
              placeholderTextColor={colors.textMuted}
              value={customMac}
              onChangeText={setCustomMac}
              autoCapitalize="characters"
            />
            <PrimaryButton
              title="Ghép"
              onPress={handleManualPair}
              disabled={!customMac.trim() || pairingMac !== null}
              style={styles.manualBtn}
            />
          </View>
        </Card>
      )}

      {/* Discovered Devices List */}
      <Text style={styles.sectionHeader}>Thiết bị tìm thấy ({devices.length})</Text>

      <FlatList
        data={devices}
        keyExtractor={(item) => item.mac}
        renderItem={({ item }) => (
          <BtDeviceItem
            device={item}
            onSelect={handlePair}
            isPairing={pairingMac === item.mac}
            isPaired={isAlreadyPaired(item.mac)}
          />
        )}
        ListEmptyComponent={
          <View style={styles.emptyContainer}>
            <ActivityIndicator size="large" color={colors.accentCyan} style={styles.emptySpinner} />
            <Text style={styles.emptyText}>Đang quét tìm thiết bị...</Text>
          </View>
        }
        contentContainerStyle={styles.listContent}
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.bgPrimary,
    paddingTop: 48,
    paddingHorizontal: 16,
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 16,
  },
  backBtn: {
    padding: 8,
    marginRight: 8,
  },
  backBtnText: {
    color: colors.textPrimary,
    fontSize: 22,
    fontWeight: '700',
  },
  headerTitles: {
    flex: 1,
  },
  headerRight: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  logsBtn: {
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6,
  },
  logsBtnText: {
    color: colors.accentCyan,
    fontSize: 11,
    fontWeight: '700',
  },
  title: {
    fontSize: 18,
    fontWeight: '800',
    color: colors.textPrimary,
  },
  subtitle: {
    fontSize: 12,
    color: colors.textSecondary,
    marginTop: 2,
  },
  myDeviceCard: {
    marginBottom: 16,
    padding: 14,
    borderColor: colors.borderActive,
  },
  myDeviceRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
  },
  myDeviceLabel: {
    fontSize: 12,
    color: colors.accentCyan,
    fontWeight: '700',
    marginBottom: 2,
  },
  myDeviceMac: {
    fontSize: 14,
    color: colors.textPrimary,
    fontFamily: 'monospace',
    fontWeight: '600',
  },
  myDeviceBtn: {
    minWidth: 100,
  },
  manualCard: {
    marginBottom: 16,
    padding: 14,
  },
  manualLabel: {
    fontSize: 12,
    color: colors.textSecondary,
    marginBottom: 8,
    fontWeight: '600',
  },
  manualRow: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  input: {
    flex: 1,
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 8,
    color: colors.textPrimary,
    fontSize: 13,
    fontFamily: 'monospace',
    marginRight: 8,
  },
  manualBtn: {
    minWidth: 80,
  },
  sectionHeader: {
    fontSize: 13,
    fontWeight: '700',
    color: colors.textSecondary,
    marginBottom: 10,
    textTransform: 'uppercase',
    letterSpacing: 0.5,
  },
  listContent: {
    paddingBottom: 24,
  },
  emptyContainer: {
    paddingVertical: 40,
    alignItems: 'center',
  },
  emptySpinner: {
    marginBottom: 12,
  },
  emptyText: {
    color: colors.textMuted,
    fontSize: 13,
  },
  pinCard: {
    marginBottom: 16,
    padding: 16,
    alignItems: 'center',
    backgroundColor: 'rgba(234, 179, 8, 0.12)',
    borderColor: '#eab308',
    borderWidth: 2,
    borderRadius: 12,
  },
  pinLabel: {
    fontSize: 12,
    fontWeight: '800',
    color: '#eab308',
    letterSpacing: 1,
    marginBottom: 4,
  },
  pinValue: {
    fontSize: 32,
    fontWeight: '900',
    color: colors.textPrimary,
    letterSpacing: 4,
    fontFamily: 'monospace',
    marginVertical: 4,
  },
  pinSub: {
    fontSize: 12,
    color: colors.textSecondary,
    textAlign: 'center',
    marginTop: 2,
  },
});
