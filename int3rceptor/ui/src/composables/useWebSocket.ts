import { ref, onUnmounted } from 'vue';
import type { CaptureEntry } from '@/types';

export function useWebSocket(url: string) {
  const requests = ref<CaptureEntry[]>([]);
  const latest = ref<CaptureEntry | null>(null);
  const connected = ref(false);
  const ws = new WebSocket(url);

  ws.onopen = () => {
    connected.value = true;
  };

  ws.onmessage = (event) => {
    try {
      const payload = JSON.parse(event.data);
      if (payload.type === 'request' && payload.data) {
        const entry = payload.data as CaptureEntry;
        latest.value = entry;
        requests.value = [entry, ...requests.value].slice(0, 100);
      }
    } catch (err) {
      console.error('Failed to parse WS message', err);
    }
  };

  ws.onclose = () => {
    connected.value = false;
  };

  onUnmounted(() => ws.close());

  return { requests, connected, latest };
}
