export interface GpsTelemetry {
  spd: number;
  lat: number;
  lon: number;
  alt: number;
  hdg: number;
  accuracy?: number;
}

export interface TelemetryState {
  liveSpeedKmh: number;
  accuracyM: number | null;
  altitudeM: number | null;
  sendRateHz: number;
  sendCount: number;
  lastSendTime: number;
}
