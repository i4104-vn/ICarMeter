import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { colors } from '../../theme/colors';
import { typography } from '../../theme/typography';
import { useApp } from '../../context/AppContext';

export const AppHeader: React.FC = () => {
  const { isWifiConnected, forceReconnectWifi } = useApp();

  return (
    <View style={styles.header}>
      <View style={styles.headerLeft}>
        <View style={styles.logoBadge}>
          <Text style={styles.logoText}>ICAR</Text>
        </View>
        <View style={styles.titleWrap}>
          <Text style={styles.title}>ICarMeter</Text>
          <Text style={styles.subtitle}>Wi-Fi Companion</Text>
        </View>
      </View>

      <TouchableOpacity
        activeOpacity={0.7}
        onPress={forceReconnectWifi}
        style={[
          styles.statusBadge,
          isWifiConnected ? styles.statusBadgeConnected : styles.statusBadgeDisconnected,
        ]}
      >
        <View
          style={[
            styles.statusDot,
            isWifiConnected ? styles.statusDotConnected : styles.statusDotDisconnected,
          ]}
        />
        <Text style={styles.statusText}>
          {isWifiConnected ? 'Đã kết nối' : 'Chưa kết nối'}
        </Text>
      </TouchableOpacity>
    </View>
  );
};

const styles = StyleSheet.create({
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
    backgroundColor: colors.bgSurface,
    borderBottomWidth: 1,
    borderBottomColor: colors.borderSubtle,
  },
  headerLeft: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  logoBadge: {
    backgroundColor: 'rgba(56, 189, 248, 0.15)',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 6,
    borderWidth: 1,
    borderColor: 'rgba(56, 189, 248, 0.4)',
  },
  logoText: {
    color: colors.accentCyan,
    fontSize: 12,
    fontWeight: '900',
    letterSpacing: 1,
  },
  titleWrap: {
    justifyContent: 'center',
  },
  title: {
    ...typography.h2,
    color: colors.textPrimary,
  },
  subtitle: {
    ...typography.bodySmall,
    color: colors.textSecondary,
  },
  statusBadge: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 6,
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderRadius: 20,
    borderWidth: 1,
  },
  statusBadgeConnected: {
    backgroundColor: 'rgba(34, 197, 94, 0.12)',
    borderColor: 'rgba(34, 197, 94, 0.35)',
  },
  statusBadgeDisconnected: {
    backgroundColor: 'rgba(148, 163, 184, 0.1)',
    borderColor: 'rgba(148, 163, 184, 0.25)',
  },
  statusDot: {
    width: 8,
    height: 8,
    borderRadius: 4,
  },
  statusDotConnected: {
    backgroundColor: colors.statusConnected,
  },
  statusDotDisconnected: {
    backgroundColor: colors.statusDisconnected,
  },
  statusText: {
    fontSize: 12,
    fontWeight: '600',
    color: colors.textPrimary,
  },
});
