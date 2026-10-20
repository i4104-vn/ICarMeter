import React from 'react';
import { View, Text, Switch, StyleSheet, ViewStyle } from 'react-native';
import { colors } from '../../theme/colors';

interface CustomSwitchProps {
  title: string;
  description?: string;
  value: boolean;
  onValueChange: (val: boolean) => void;
  style?: ViewStyle;
}

export const CustomSwitch: React.FC<CustomSwitchProps> = ({
  title,
  description,
  value,
  onValueChange,
  style,
}) => {
  return (
    <View style={[styles.container, style]}>
      <View style={styles.textWrap}>
        <Text style={styles.title}>{title}</Text>
        {description ? <Text style={styles.description}>{description}</Text> : null}
      </View>
      <Switch
        value={value}
        onValueChange={onValueChange}
        trackColor={{ false: '#334155', true: '#0284c7' }}
        thumbColor={value ? colors.accentCyan : '#94a3b8'}
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingVertical: 10,
  },
  textWrap: {
    flex: 1,
    paddingRight: 16,
  },
  title: {
    fontSize: 14,
    fontWeight: '600',
    color: colors.textPrimary,
  },
  description: {
    fontSize: 12,
    color: colors.textSecondary,
    marginTop: 2,
    lineHeight: 16,
  },
});
