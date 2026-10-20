import { TransportClient } from './transport';
import { WsClient } from './wsClient';
import { WsClientCommand, WsServerMessage } from '../types/websocket';

export class WifiWsTransport implements TransportClient {
  readonly type = 'wifi-ws' as const;
  private wsClient: WsClient;

  constructor(host = '192.168.4.1:8080') {
    this.wsClient = new WsClient(host);
  }

  get connected(): boolean {
    return this.wsClient.connected;
  }

  setHost(host: string) {
    this.wsClient.setHost(host);
  }

  start(): void {
    this.wsClient.start();
  }

  stop(): void {
    this.wsClient.stop();
  }

  send(cmd: WsClientCommand): boolean {
    if (cmd.type === 'get_config' || cmd.type === 'set_config' || cmd.type === 'telemetry') {
      console.warn('[WiFi-WS] Data operations are strictly forbidden over WiFi');
      return false;
    }
    return this.wsClient.send(cmd);
  }

  onConnectionChange(cb: (connected: boolean) => void): () => void {
    return this.wsClient.onConnectionChange(cb);
  }

  onMessage(cb: (msg: WsServerMessage) => void): () => void {
    return this.wsClient.onMessage(cb);
  }
}
