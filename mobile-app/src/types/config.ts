export interface TpmsConfig {
  fl: string;
  fr: string;
  rl: string;
  rr: string;
  alert: number;
  high_temp_alert: number;
}

export interface GpsConfig {
  enabled: boolean;
}

export interface PiFullConfig {
  tpms: TpmsConfig;
  gps: GpsConfig;
}

export interface AppLocalSettings {
  piHost: string;
  phoneMac: string;
  sendRateHz: number;
  backgroundServiceEnabled: boolean;
}
