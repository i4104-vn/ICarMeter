import { WsClient } from './wsClient';
import { fetchWithRetry, resilientFetch } from './httpClient';
import { PiFullConfig, TpmsConfig } from '../types/config';
import { GpsTelemetry } from '../types/telemetry';

export class PiApiService {
  static async fetchConfig(wsClient: WsClient, host: string): Promise<PiFullConfig | null> {
    if (wsClient.connected) {
      wsClient.send({ type: 'get_config' });
      // WsClient handler will receive message and update state
      return null;
    }

    // HTTP Fallback
    const res = await fetchWithRetry(`http://${host}/api/config`, { timeoutMs: 3500 }, 2);
    if (res.ok && res.data) {
      return res.data;
    }
    return null;
  }

  static async saveConfig(
    wsClient: WsClient,
    host: string,
    tpms: TpmsConfig,
    gpsEnabled: boolean
  ): Promise<boolean> {
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

    if (wsClient.connected) {
      return wsClient.send({
        type: 'set_config',
        payload,
      });
    }

    // Fallback: HTTP POST
    const res = await fetchWithRetry(
      `http://${host}/api/config`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
        timeoutMs: 4000,
      },
      2
    );

    return res.ok;
  }

  static async sendTelemetry(
    wsClient: WsClient,
    host: string,
    telemetry: GpsTelemetry
  ): Promise<boolean> {
    if (wsClient.connected) {
      return wsClient.send({
        type: 'telemetry',
        spd: telemetry.spd,
        lat: telemetry.lat,
        lon: telemetry.lon,
        alt: telemetry.alt,
        hdg: telemetry.hdg,
      });
    }

    // Fallback: HTTP POST
    const res = await resilientFetch(`http://${host}/api/gps`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(telemetry),
      timeoutMs: 1500,
    });

    return res.ok;
  }
}
