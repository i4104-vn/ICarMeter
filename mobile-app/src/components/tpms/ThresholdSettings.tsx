import React from 'react';
import { View, Text, TextInput, StyleSheet } from 'react-native';
import { PrimaryButton } from '../common/PrimaryButton';
import { useApp } from '../../context/AppContext';
import { colors } from '../../theme/colors';

export const ThresholdSettings: React.FC = () => {
  const { config, updateTpmsField, reloadConfigFromPi, saveAndSyncConfig } = useApp();

  return (
    <View style={styles.container}>
      <View style={styles.row}>
        <View style={styles.labelWrap}>
          <Text style={styles.title}>Áp suất tối thiểu (Bar)</Text>
          <Text style={styles.desc}>Báo động khi dưới ngưỡng</Text>
        </View>
        <TextInput
          style={styles.numInput}
          value={config.tpms.alert.toString()}
          onChangeText={(txt) => {
            const num = parseFloat(txt);
            updateTpmsField('alert', isNaN(num) ? 0 : num);
          }}
          keyboardType="decimal-pad"
          selectTextOnFocus
        />
      </View>

      <View style={styles.row}>
        <View style={styles.labelWrap}>
          <Text style={styles.title}>Nhiệt độ tối đa (°C)</Text>
          <Text style={styles.desc}>Báo động khi vượt ngưỡng</Text>
        </View>
        <TextInput
          style={styles.numInput}
          value={config.tpms.high_temp_alert.toString()}
          onChangeText={(txt) => {
            const num = parseFloat(txt);
            updateTpmsField('high_temp_alert', isNaN(num) ? 0 : num);
          }}
          keyboardType="numeric"
          selectTextOnFocus
        />
      </View>

      <View style={styles.btnRow}>
        <PrimaryButton
          title="Tải lại"
          variant="secondary"
          onPress={reloadConfigFromPi}
          style={styles.btnFlex}
        />
        <PrimaryButton
          title="Lưu cấu hình"
          variant="primary"
          onPress={() => saveAndSyncConfig()}
          style={styles.btnFlex}
        />
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    marginTop: 14,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: colors.borderSubtle,
  },
  row: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 8,
  },
  labelWrap: {
    flex: 1,
    paddingRight: 12,
  },
  title: {
    fontSize: 13,
    fontWeight: '600',
    color: colors.textPrimary,
  },
  desc: {
    fontSize: 11,
    color: colors.textSecondary,
    marginTop: 2,
  },
  numInput: {
    width: 72,
    height: 38,
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 8,
    textAlign: 'center',
    color: colors.textPrimary,
    fontSize: 14,
    fontWeight: '700',
  },
  btnRow: {
    flexDirection: 'row',
    gap: 10,
    marginTop: 14,
  },
  btnFlex: {
    flex: 1,
  },
});
