import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';
import { colors } from '../../theme/colors';
import { BtDevice } from '../../types/bluetooth';

interface BtDeviceItemProps {
  device: BtDevice;
  onSelect: (mac: string) => void;
  isPairing: boolean;
  isPaired?: boolean;
}

export const BtDeviceItem: React.FC<BtDeviceItemProps> = ({
  device,
  onSelect,
  isPairing,
  isPaired = false,
}) => {
  return (
    <View style={styles.container}>
      <View style={styles.infoCol}>
        <View style={styles.row}>
          <Text style={styles.name} numberOfLines={1}>
            {device.name || device.mac}
          </Text>
          {typeof device.rssi === 'number' && device.rssi !== 0 && (
            <Text style={styles.rssi}>{device.rssi} dBm</Text>
          )}
        </View>
        <Text style={styles.mac}>{device.mac}</Text>
      </View>

      <TouchableOpacity
        style={[
          styles.actionBtn,
          isPaired ? styles.btnPaired : styles.btnPair,
          isPairing && styles.btnDisabled,
        ]}
        disabled={isPairing || isPaired}
        onPress={() => onSelect(device.mac)}
        activeOpacity={0.8}
      >
        <Text style={[styles.btnText, isPaired ? styles.textPaired : styles.textPair]}>
          {isPaired ? 'Đã ghép' : isPairing ? 'Đang ghép...' : 'Ghép đôi'}
        </Text>
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    backgroundColor: colors.bgCard,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 12,
    padding: 14,
    marginBottom: 10,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  infoCol: {
    flex: 1,
    marginRight: 12,
  },
  row: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 4,
  },
  name: {
    color: colors.textPrimary,
    fontSize: 15,
    fontWeight: '700',
    flexShrink: 1,
  },
  rssi: {
    color: colors.accentCyan,
    fontSize: 11,
    marginLeft: 8,
    fontWeight: '600',
  },
  mac: {
    color: colors.textMuted,
    fontSize: 12,
    fontFamily: 'monospace',
  },
  actionBtn: {
    paddingVertical: 8,
    paddingHorizontal: 14,
    borderRadius: 8,
    minWidth: 80,
    alignItems: 'center',
    justifyContent: 'center',
  },
  btnPair: {
    backgroundColor: colors.accentCyan,
  },
  btnPaired: {
    backgroundColor: 'rgba(34, 197, 94, 0.15)',
    borderWidth: 1,
    borderColor: colors.accentEmerald,
  },
  btnDisabled: {
    opacity: 0.6,
  },
  btnText: {
    fontSize: 13,
    fontWeight: '700',
  },
  textPair: {
    color: '#0f172a',
  },
  textPaired: {
    color: colors.accentEmerald,
  },
});
