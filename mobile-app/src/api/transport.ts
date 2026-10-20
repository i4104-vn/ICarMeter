import { WsClientCommand, WsServerMessage } from '../types/websocket';

export type TransportType = 'wifi-ws' | 'bt-spp';

export interface TransportClient {
  readonly connected: boolean;
  readonly type: TransportType;

  start(): void;
  stop(): void;
  send(cmd: WsClientCommand): boolean;

  onConnectionChange(cb: (connected: boolean) => void): () => void;
  onMessage(cb: (msg: WsServerMessage) => void): () => void;
}
