import React, { useState } from 'react';
import { View, Text, TextInput, StyleSheet } from 'react-native';
import { Card } from '../common/Card';
import { PrimaryButton } from '../common/PrimaryButton';
import { useApp } from '../../context/AppContext';
import { colors } from '../../theme/colors';

export const AutoTestCard: React.FC = () => {
  const { settings, setPhoneMac, autoTest, startAutoTest, stopAutoTest } = useApp();
  const [testPressure, setTestPressure] = useState('2.4');
  const [testTemp, setTestTemp] = useState('75');

  const getBadgeStyle = () => {
    switch (autoTest.statusBadge) {
      case 'ĐANG TEST':
        return styles.badgeRunning;
      case 'HOÀN TẤT':
        return styles.badgeDone;
      default:
        return styles.badgeReady;
    }
  };

  return (
    <Card
      label="TEST CẢM BIẾN (BLE)"
      badge={
        <View style={[styles.badge, getBadgeStyle()]}>
          <Text style={styles.badgeText}>{autoTest.statusBadge}</Text>
        </View>
      }
    >
      <View style={styles.formGroup}>
        <Text style={styles.fieldLabel}>MAC Bluetooth</Text>
        <TextInput
          style={styles.textInput}
          value={settings.phoneMac}
          onChangeText={(txt) => setPhoneMac(txt)}
          placeholder="AA:BB:CC:11:22:33"
          placeholderTextColor={colors.textMuted}
          autoCapitalize="characters"
          autoCorrect={false}
          maxLength={17}
        />
      </View>

      <View style={styles.paramsRow}>
        <View style={styles.paramItem}>
          <Text style={styles.fieldLabel}>Áp suất (Bar)</Text>
          <TextInput
            style={styles.numInput}
            value={testPressure}
            onChangeText={setTestPressure}
            keyboardType="decimal-pad"
            selectTextOnFocus
          />
        </View>

        <View style={styles.paramItem}>
          <Text style={styles.fieldLabel}>Nhiệt độ (°C)</Text>
          <TextInput
            style={styles.numInput}
            value={testTemp}
            onChangeText={setTestTemp}
            keyboardType="numeric"
            selectTextOnFocus
          />
        </View>
      </View>

      {autoTest.isRunning && (
        <View style={styles.progressBox}>
          <Text style={styles.stepTitle}>{autoTest.stepTitle}</Text>
          <Text style={styles.stepDesc}>{autoTest.stepDesc}</Text>
          <View style={styles.progressBarTrack}>
            <View style={[styles.progressBarFill, { width: `${autoTest.percent}%` }]} />
          </View>
        </View>
      )}

      <View style={styles.actionsRow}>
        {autoTest.isRunning ? (
          <PrimaryButton
            title="Dừng"
            variant="danger"
            onPress={stopAutoTest}
            style={styles.btnFull}
          />
        ) : (
          <PrimaryButton
            title="Bắt đầu test"
            variant="primary"
            onPress={() => {
              const p = parseFloat(testPressure) || 2.4;
              const t = parseFloat(testTemp) || 75;
              startAutoTest(p, t);
            }}
            style={styles.btnFull}
          />
        )}
      </View>
    </Card>
  );
};

const styles = StyleSheet.create({
  badge: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 6,
    borderWidth: 1,
  },
  badgeReady: {
    backgroundColor: 'rgba(56, 189, 248, 0.15)',
    borderColor: 'rgba(56, 189, 248, 0.4)',
  },
  badgeRunning: {
    backgroundColor: 'rgba(234, 179, 8, 0.15)',
    borderColor: 'rgba(234, 179, 8, 0.5)',
  },
  badgeDone: {
    backgroundColor: 'rgba(34, 197, 94, 0.15)',
    borderColor: 'rgba(34, 197, 94, 0.5)',
  },
  badgeText: {
    fontSize: 10,
    fontWeight: '800',
    color: colors.textPrimary,
  },
  formGroup: {
    marginBottom: 10,
  },
  fieldLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: colors.textSecondary,
    marginBottom: 6,
  },
  textInput: {
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 8,
    color: colors.textPrimary,
    fontSize: 13,
    fontFamily: 'monospace',
    fontWeight: '700',
  },
  paramsRow: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 12,
  },
  paramItem: {
    flex: 1,
  },
  numInput: {
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 8,
    paddingHorizontal: 12,
    paddingVertical: 8,
    color: colors.textPrimary,
    fontSize: 14,
    fontWeight: '700',
    textAlign: 'center',
  },
  progressBox: {
    backgroundColor: 'rgba(0, 0, 0, 0.25)',
    borderRadius: 8,
    padding: 12,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: colors.border,
  },
  stepTitle: {
    fontSize: 13,
    fontWeight: '700',
    color: colors.accentCyan,
    marginBottom: 4,
  },
  stepDesc: {
    fontSize: 11,
    color: colors.textSecondary,
    marginBottom: 8,
  },
  progressBarTrack: {
    height: 6,
    backgroundColor: '#334155',
    borderRadius: 3,
    overflow: 'hidden',
  },
  progressBarFill: {
    height: '100%',
    backgroundColor: colors.accentCyan,
  },
  actionsRow: {
    marginTop: 2,
  },
  btnFull: {
    width: '100%',
  },
});
