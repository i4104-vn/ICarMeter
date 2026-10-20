import { NativeICarMeter } from '../native/ICarMeterNative';

interface FetchOptions extends RequestInit {
  timeoutMs?: number;
}

export async function resilientFetch(url: string, options: FetchOptions = {}): Promise<{ ok: boolean; status: number; data?: any; text?: string; error?: string }> {
  const method = (options.method || 'GET').toUpperCase();
  const bodyStr = options.body ? (typeof options.body === 'string' ? options.body : JSON.stringify(options.body)) : '';
  const timeoutMs = options.timeoutMs || 3500;

  // Try Native Bridge HTTP first (guarantees Wi-Fi bound socket)
  try {
    const nativeRes = method === 'POST'
      ? await NativeICarMeter.nativeHttpPost(url, bodyStr)
      : await NativeICarMeter.nativeHttpGet(url);

    if (nativeRes.status >= 200 && nativeRes.status < 300) {
      let parsed = {};
      try {
        if (nativeRes.body) parsed = JSON.parse(nativeRes.body);
      } catch {
        // text body
      }
      return { ok: true, status: nativeRes.status, data: parsed, text: nativeRes.body };
    } else if (nativeRes.status > 0) {
      return { ok: false, status: nativeRes.status, error: nativeRes.error || `HTTP ${nativeRes.status}` };
    }
  } catch {
    // Fall back to standard fetch
  }

  // Standard fetch with timeout
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const res = await fetch(url, {
      ...options,
      signal: controller.signal,
    });
    clearTimeout(timer);

    let parsed = null;
    const text = await res.text();
    try {
      parsed = JSON.parse(text);
    } catch {
      // ignore
    }

    return {
      ok: res.ok,
      status: res.status,
      data: parsed,
      text,
      error: res.ok ? undefined : `HTTP ${res.status}`,
    };
  } catch (err: any) {
    clearTimeout(timer);
    return {
      ok: false,
      status: 0,
      error: err?.message || 'Network request failed',
    };
  }
}

export async function fetchWithRetry(url: string, options: FetchOptions = {}, maxRetries = 2) {
  let lastRes = await resilientFetch(url, options);
  if (lastRes.ok) return lastRes;

  for (let i = 0; i < maxRetries; i++) {
    await new Promise((r) => setTimeout(r, 400 * (i + 1)));
    lastRes = await resilientFetch(url, options);
    if (lastRes.ok) return lastRes;
  }

  return lastRes;
}
