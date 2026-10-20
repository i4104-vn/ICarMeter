import { NativeICarMeter } from '../native/ICarMeterNative';
import { TpmsConfig } from '../types/config';

export interface AutoTestProgress {
  isRunning: boolean;
  stepTitle: string;
  stepDesc: string;
  percent: number;
  statusBadge: 'SẴN SÀNG' | 'ĐANG TEST' | 'HOÀN TẤT';
}

export type AutoTestProgressListener = (progress: AutoTestProgress) => void;

export class BleTpmsAutoTester {
  private isRunning = false;
  private originalConfig: TpmsConfig | null = null;
  private listener: AutoTestProgressListener | null = null;

  subscribe(listener: AutoTestProgressListener): () => void {
    this.listener = listener;
    this.notify({
      isRunning: this.isRunning,
      stepTitle: 'Sẵn sàng',
      stepDesc: 'Nhấn Bắt Đầu để kiểm tra 4 van cảm biến qua sóng BLE',
      percent: 0,
      statusBadge: 'SẴN SÀNG',
    });
    return () => {
      this.listener = null;
    };
  }

  private notify(progress: AutoTestProgress) {
    if (this.listener) {
      this.listener(progress);
    }
  }

  async runAutoTest(
    phoneMac: string,
    testPressure: number,
    testTemp: number,
    currentConfig: TpmsConfig,
    syncConfigToPi: (config: TpmsConfig) => Promise<boolean>
  ): Promise<void> {
    if (this.isRunning) return;

    this.isRunning = true;
    this.originalConfig = { ...currentConfig };

    const steps: { key: keyof TpmsConfig; label: string; percent: number }[] = [
      { key: 'fl', label: 'Trước Trái (FL)', percent: 25 },
      { key: 'fr', label: 'Trước Phải (FR)', percent: 50 },
      { key: 'rl', label: 'Sau Trái (RL)', percent: 75 },
      { key: 'rr', label: 'Sau Phải (RR)', percent: 100 },
    ];

    for (let i = 0; i < steps.length; i++) {
      if (!this.isRunning) break;

      const step = steps[i];

      // 1. Point wheel ID to phoneMac
      const stepConfig: TpmsConfig = {
        ...this.originalConfig,
        [step.key]: phoneMac,
      };

      await syncConfigToPi(stepConfig);

      // 2. Start BLE Advertising over the air
      await NativeICarMeter.startBleAdvertising(phoneMac, testPressure, testTemp);

      // 3. Countdown 5s
      for (let countdown = 5; countdown > 0; countdown--) {
        if (!this.isRunning) break;
        this.notify({
          isRunning: true,
          stepTitle: `Kiểm tra ${step.label} (${countdown}s)`,
          stepDesc: `Phát BLE ${testPressure} bar, ${testTemp}°C`,
          percent: step.percent,
          statusBadge: 'ĐANG TEST',
        });
        await new Promise((r) => setTimeout(r, 1000));
      }

      // Stop BLE Advertising for this step
      await NativeICarMeter.stopBleAdvertising();
    }

    // Restore original config
    await this.restoreOriginal(syncConfigToPi);

    this.notify({
      isRunning: false,
      stepTitle: 'Hoàn tất test',
      stepDesc: 'Đã kiểm tra 4 bánh và khôi phục ID gốc.',
      percent: 100,
      statusBadge: 'HOÀN TẤT',
    });
  }

  async stopAutoTest(syncConfigToPi: (config: TpmsConfig) => Promise<boolean>): Promise<void> {
    if (!this.isRunning) return;
    this.isRunning = false;
    await NativeICarMeter.stopBleAdvertising();
    await this.restoreOriginal(syncConfigToPi);
    this.notify({
      isRunning: false,
      stepTitle: 'Đã dừng test',
      stepDesc: 'Đã khôi phục ID gốc.',
      percent: 0,
      statusBadge: 'SẴN SÀNG',
    });
  }

  private async restoreOriginal(syncConfigToPi: (config: TpmsConfig) => Promise<boolean>) {
    if (this.originalConfig) {
      await syncConfigToPi(this.originalConfig);
      this.originalConfig = null;
    }
    this.isRunning = false;
  }
}
