import React, { useEffect, useState } from 'react';
import { View, Text, StyleSheet, ActivityIndicator, TouchableOpacity } from 'react-native';
import { colors } from '../theme/colors';
import { Card } from '../components/common/Card';
import { PrimaryButton } from '../components/common/PrimaryButton';
import { useApp } from '../context/AppContext';
import { NativeICarMeter } from '../native/ICarMeterNative';

interface WifiSetupScreenProps {
  onWifiReady: () => void;
}

export const WifiSetupScreen: React.FC<WifiSetupScreenProps> = ({ onWifiReady }) => {
  const { isWifiConnected, wifiTransport, setPhoneMac } = useApp();
  const [connecting, setConnecting] = useState(false);
  const [statusText, setStatusText] = useState('Chưa kết nối');

  useEffect(() => {
    // Auto-detect Bluetooth address of this phone
    async function detectBtMac() {
      const mac = await NativeICarMeter.getDeviceBluetoothAddress();
      if (mac) {
        await setPhoneMac(mac);
      }
    }
    detectBtMac();
  }, [setPhoneMac]);

  useEffect(() => {
    if (isWifiConnected) {
      setConnecting(false);
      setStatusText('Đã kết nối WiFi');
      const timer = setTimeout(() => {
        onWifiReady();
      }, 500);
      return () => clearTimeout(timer);
    }
  }, [isWifiConnected, onWifiReady]);

  const handleConnectWifi = async () => {
    setConnecting(true);
    setStatusText('Đang kết nối WiFi xe...');
    try {
      await NativeICarMeter.bindToCarWifi();
      wifiTransport.start();
    } catch {
      setStatusText('Không thể kết nối. Vui lòng kiểm tra WiFi');
      setConnecting(false);
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.content}>
        <View style={styles.iconCircle}>
          <Text style={styles.icon}>📡</Text>
        </View>

        <Text style={styles.title}>Thiết Lập Bluetooth</Text>
        <Text style={styles.subtitle}>
          Kết nối WiFi của xe để cài đặt Bluetooth lần đầu.
        </Text>

        <Card style={styles.guideCard}>
          <View style={styles.stepRow}>
            <View style={styles.stepNum}>
              <Text style={styles.stepNumText}>1</Text>
            </View>
            <Text style={styles.stepText}>Vào Cài đặt → Bật Wi-Fi</Text>
          </View>

          <View style={styles.stepRow}>
            <View style={styles.stepNum}>
              <Text style={styles.stepNumText}>2</Text>
            </View>
            <Text style={styles.stepText}>Chọn mạng "ICarMeter"</Text>
          </View>

          <View style={styles.stepRow}>
            <View style={styles.stepNum}>
              <Text style={styles.stepNumText}>3</Text>
            </View>
            <Text style={styles.stepText}>Quay lại và nhấn nút bên dưới</Text>
          </View>
        </Card>

        <View style={styles.statusBox}>
          {connecting ? (
            <ActivityIndicator size="small" color={colors.accentCyan} style={styles.spinner} />
          ) : (
            <View
              style={[
                styles.dot,
                { backgroundColor: isWifiConnected ? colors.statusConnected : colors.statusDisconnected },
              ]}
            />
          )}
          <Text style={styles.statusBoxText}>{statusText}</Text>
        </View>

        <PrimaryButton
          title={connecting ? 'Đang kết nối...' : 'Đã kết nối WiFi'}
          onPress={handleConnectWifi}
          disabled={connecting}
          style={styles.connectBtn}
        />
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: colors.bgPrimary,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  content: {
    width: '100%',
    maxWidth: 420,
    alignItems: 'center',
  },
  iconCircle: {
    width: 64,
    height: 64,
    borderRadius: 32,
    backgroundColor: 'rgba(56, 189, 248, 0.1)',
    borderWidth: 1,
    borderColor: 'rgba(56, 189, 248, 0.25)',
    justifyContent: 'center',
    alignItems: 'center',
    marginBottom: 16,
  },
  icon: {
    fontSize: 28,
  },
  title: {
    fontSize: 20,
    fontWeight: '800',
    color: colors.textPrimary,
    marginBottom: 6,
  },
  subtitle: {
    fontSize: 13,
    color: colors.textSecondary,
    textAlign: 'center',
    marginBottom: 20,
    lineHeight: 18,
  },
  guideCard: {
    width: '100%',
    padding: 16,
    marginBottom: 16,
  },
  stepRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 6,
  },
  stepNum: {
    width: 22,
    height: 22,
    borderRadius: 11,
    backgroundColor: 'rgba(56, 189, 248, 0.15)',
    justifyContent: 'center',
    alignItems: 'center',
    marginRight: 10,
  },
  stepNumText: {
    color: colors.accentCyan,
    fontSize: 12,
    fontWeight: '700',
  },
  stepText: {
    color: colors.textPrimary,
    fontSize: 13,
    fontWeight: '500',
  },
  statusBox: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 20,
  },
  dot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    marginRight: 8,
  },
  spinner: {
    marginRight: 8,
  },
  statusBoxText: {
    color: colors.textMuted,
    fontSize: 13,
  },
  connectBtn: {
    width: '100%',
  },
});
