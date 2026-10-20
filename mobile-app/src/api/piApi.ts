import { TransportClient } from './transport';
import { PiFullConfig, TpmsConfig } from '../types/config';
import { GpsTelemetry } from '../types/telemetry';

export class PiApiService {
  /**
   * Requests full configuration from Pi over Bluetooth transport.
   */
  static fetchConfig(transport: TransportClient): boolean {
    if (!transport.connected) {
      return false;
    }
    return transport.send({ type: 'get_config' });
  }

  /**
   * Saves updated configuration to Pi over Bluetooth transport.
   */
  static saveConfig(
    transport: TransportClient,
    tpms: TpmsConfig,
    gpsEnabled: boolean
  ): boolean {
    if (!transport.connected) {
      return false;
    }

    const payload = {
      action: 'update_config' as const,
      fl: tpms.fl.trim().toUpperCase(),
      fr: tpms.fr.trim().toUpperCase(),
      rl: tpms.rl.trim().toUpperCase(),
      rr: tpms.rr.trim().toUpperCase(),
      alert: tpms.alert,
      high_temp_alert: tpms.high_temp_alert,
      gps_enabled: gpsEnabled,
    };

    return transport.send({
      type: 'set_config',
      payload,
    });
  }

  /**
   * Transmits GPS telemetry to Pi over Bluetooth transport.
   */
  static sendTelemetry(
    transport: TransportClient,
    telemetry: GpsTelemetry
  ): boolean {
    if (!transport.connected) {
      return false;
    }

    return transport.send({
      type: 'telemetry',
      spd: telemetry.spd,
      lat: telemetry.lat,
      lon: telemetry.lon,
      alt: telemetry.alt,
      hdg: telemetry.hdg,
    });
  }
}
