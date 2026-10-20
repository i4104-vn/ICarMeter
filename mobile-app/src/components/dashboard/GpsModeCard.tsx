import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { Card } from '../common/Card';
import { CustomSwitch } from '../common/CustomSwitch';
import { useApp } from '../../context/AppContext';
import { colors } from '../../theme/colors';

export const GpsModeCard: React.FC = () => {
  const { config, updateGpsHardwareToggle } = useApp();
  const isHardware = config.gps.enabled;

  return (
    <Card
      label="ĐỊNH VỊ GPS"
      badge={
        <View
          style={[
            styles.modeBadge,
            isHardware ? styles.modeBadgeHw : styles.modeBadgePhone,
          ]}
        >
          <Text style={styles.modeBadgeText}>
            {isHardware ? 'MODULE XE' : 'ĐIỆN THOẠI'}
          </Text>
        </View>
      }
    >
      <CustomSwitch
        title="Dùng GPS phần cứng"
        description={isHardware ? 'Lấy tốc độ từ module GPS trên xe' : 'Truyền tốc độ điện thoại qua Wi-Fi'}
        value={isHardware}
        onValueChange={(val) => updateGpsHardwareToggle(val)}
      />
    </Card>
  );
};

const styles = StyleSheet.create({
  modeBadge: {
    paddingHorizontal: 8,
    paddingVertical: 3,
    borderRadius: 6,
    borderWidth: 1,
  },
  modeBadgeHw: {
    backgroundColor: 'rgba(34, 197, 94, 0.15)',
    borderColor: 'rgba(34, 197, 94, 0.4)',
  },
  modeBadgePhone: {
    backgroundColor: 'rgba(56, 189, 248, 0.15)',
    borderColor: 'rgba(56, 189, 248, 0.4)',
  },
  modeBadgeText: {
    fontSize: 10,
    fontWeight: '800',
    color: colors.textPrimary,
  },
});
