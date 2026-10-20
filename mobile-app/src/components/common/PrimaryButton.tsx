import React from 'react';
import { TouchableOpacity, Text, StyleSheet, ViewStyle, TextStyle } from 'react-native';
import { colors } from '../../theme/colors';

interface ButtonProps {
  title: string;
  onPress: () => void;
  variant?: 'primary' | 'secondary' | 'success' | 'danger';
  style?: ViewStyle;
  textStyle?: TextStyle;
  disabled?: boolean;
}

export const PrimaryButton: React.FC<ButtonProps> = ({
  title,
  onPress,
  variant = 'primary',
  style,
  textStyle,
  disabled = false,
}) => {
  const getVariantStyle = () => {
    switch (variant) {
      case 'success':
        return styles.btnSuccess;
      case 'danger':
        return styles.btnDanger;
      case 'secondary':
        return styles.btnSecondary;
      case 'primary':
      default:
        return styles.btnPrimary;
    }
  };

  const getTextStyle = () => {
    switch (variant) {
      case 'secondary':
        return styles.textSecondary;
      default:
        return styles.textDefault;
    }
  };

  return (
    <TouchableOpacity
      activeOpacity={0.8}
      onPress={onPress}
      disabled={disabled}
      style={[styles.btnBase, getVariantStyle(), disabled && styles.btnDisabled, style]}
    >
      <Text style={[styles.btnText, getTextStyle(), textStyle]}>{title}</Text>
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  btnBase: {
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderRadius: 12,
    alignItems: 'center',
    justifyContent: 'center',
  },
  btnPrimary: {
    backgroundColor: colors.accentCyan,
  },
  btnSuccess: {
    backgroundColor: colors.accentEmerald,
  },
  btnDanger: {
    backgroundColor: colors.accentRose,
  },
  btnSecondary: {
    backgroundColor: colors.bgSurface,
    borderWidth: 1,
    borderColor: colors.border,
  },
  btnDisabled: {
    opacity: 0.5,
  },
  btnText: {
    fontSize: 14,
    fontWeight: '700',
  },
  textDefault: {
    color: '#0f172a',
  },
  textSecondary: {
    color: colors.textPrimary,
  },
});
