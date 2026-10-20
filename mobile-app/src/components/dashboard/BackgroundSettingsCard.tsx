import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { Card } from '../common/Card';
import { CustomSwitch } from '../common/CustomSwitch';
import { useApp } from '../../context/AppContext';
import { colors } from '../../theme/colors';

const RATES = [
  { val: 1, label: '1 Hz' },
  { val: 5, label: '5 Hz' },
  { val: 10, label: '10 Hz' },
];

export const BackgroundSettingsCard: React.FC = () => {
  const { settings, setBackgroundServiceEnabled, setSendRateHz } = useApp();

  return (
    <Card label="CHẠY NGẦM & TẦN SỐ">
      <CustomSwitch
        title="Dịch vụ chạy ngầm"
        description="Giữ kết nối khi tắt màn hình"
        value={settings.backgroundServiceEnabled}
        onValueChange={(val) => setBackgroundServiceEnabled(val)}
      />

      <View style={styles.rateSection}>
        <Text style={styles.rateLabel}>Tần số gửi GPS</Text>
        <View style={styles.rateRow}>
          {RATES.map((r) => {
            const isSelected = settings.sendRateHz === r.val;
            return (
              <TouchableOpacity
                key={r.val}
                activeOpacity={0.8}
                onPress={() => setSendRateHz(r.val)}
                style={[
                  styles.rateBtn,
                  isSelected && styles.rateBtnSelected,
                ]}
              >
                <Text
                  style={[
                    styles.rateText,
                    isSelected && styles.rateTextSelected,
                  ]}
                >
                  {r.label}
                </Text>
              </TouchableOpacity>
            );
          })}
        </View>
      </View>
    </Card>
  );
};

const styles = StyleSheet.create({
  rateSection: {
    marginTop: 12,
    paddingTop: 10,
    borderTopWidth: 1,
    borderTopColor: colors.borderSubtle,
  },
  rateLabel: {
    fontSize: 13,
    fontWeight: '600',
    color: colors.textPrimary,
    marginBottom: 8,
  },
  rateRow: {
    flexDirection: 'row',
    gap: 8,
  },
  rateBtn: {
    flex: 1,
    paddingVertical: 9,
    borderRadius: 8,
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
    alignItems: 'center',
    justifyContent: 'center',
  },
  rateBtnSelected: {
    backgroundColor: 'rgba(56, 189, 248, 0.15)',
    borderColor: colors.accentCyan,
  },
  rateText: {
    fontSize: 12,
    fontWeight: '600',
    color: colors.textSecondary,
  },
  rateTextSelected: {
    color: colors.accentCyan,
    fontWeight: '700',
  },
});
