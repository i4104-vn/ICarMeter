import { PiFullConfig } from './config';
import { GpsTelemetry } from './telemetry';
import { BtDevice, BtStatusData } from './bluetooth';

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
  | ({ type: 'telemetry' } & GpsTelemetry)
  | { type: 'get_bt_status' }
  | { type: 'bt_start_discovery' }
  | { type: 'bt_stop_discovery' }
  | { type: 'bt_pair'; mac: string }
  | { type: 'bt_unpair'; mac: string };

export interface WsServerMessage {
  type:
    | 'welcome'
    | 'config'
    | 'error'
    | 'status'
    | 'heartbeat'
    | 'pong'
    | 'bt_status'
    | 'bt_discovery_started'
    | 'bt_discovery_stopped'
    | 'bt_device_found'
    | 'bt_pair_result'
    | 'bt_unpair_result';
  message?: string;
  config?: PiFullConfig;
  data?: PiFullConfig | BtStatusData | any;
  status?: string;
  clients?: number;
  timestamp?: number;
  mac?: string;
  name?: string;
  rssi?: number;
  success?: boolean;
}
