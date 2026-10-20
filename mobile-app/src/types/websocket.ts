import { PiFullConfig } from './config';
import { GpsTelemetry } from './telemetry';

export type WsClientCommand =
  | { type: 'ping' }
  | { type: 'get_config' }
  | {
      type: 'set_config';
      payload: {
        action: 'update_config';
        fl: string;
        fr: string;
        rl: string;
        rr: string;
        alert: number;
        high_temp_alert: number;
        gps_enabled: boolean;
      };
    }
  | ({ type: 'telemetry' } & GpsTelemetry);

export interface WsServerMessage {
  type: 'welcome' | 'config' | 'error' | 'status' | 'heartbeat' | 'pong';
  message?: string;
  config?: PiFullConfig;
  data?: PiFullConfig;
  status?: string;
  clients?: number;
  timestamp?: number;
}
