import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import { TireInput } from './TireInput';
import { useApp } from '../../context/AppContext';
import { colors } from '../../theme/colors';

export const CarTpmsDiagram: React.FC = () => {
  const { config, updateTpmsField } = useApp();

  return (
    <View style={styles.layout}>
      {/* Front Axle */}
      <View style={styles.row}>
        <TireInput
          label="TRƯỚC TRÁI (FL)"
          value={config.tpms.fl}
          onChangeText={(txt) => updateTpmsField('fl', txt)}
        />
        <View style={styles.carDirection}>
          <Text style={styles.directionText}>▲ ĐẦU XE</Text>
        </View>
        <TireInput
          label="TRƯỚC PHẢI (FR)"
          value={config.tpms.fr}
          onChangeText={(txt) => updateTpmsField('fr', txt)}
        />
      </View>

      {/* Rear Axle */}
      <View style={styles.row}>
        <TireInput
          label="SAU TRÁI (RL)"
          value={config.tpms.rl}
          onChangeText={(txt) => updateTpmsField('rl', txt)}
        />
        <View style={styles.carDirection}>
          <Text style={styles.directionText}>▼ ĐUÔI XE</Text>
        </View>
        <TireInput
          label="SAU PHẢI (RR)"
          value={config.tpms.rr}
          onChangeText={(txt) => updateTpmsField('rr', txt)}
        />
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  layout: {
    gap: 12,
    marginVertical: 4,
  },
  row: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 8,
  },
  carDirection: {
    width: 68,
    alignItems: 'center',
    justifyContent: 'center',
  },
  directionText: {
    fontSize: 9,
    fontWeight: '800',
    color: colors.textMuted,
    letterSpacing: 0.5,
  },
});
