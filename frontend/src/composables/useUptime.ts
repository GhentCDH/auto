import { ref, onUnmounted, type Ref } from 'vue';

export interface HeartbeatEntry {
  status: number; // 1 = up, 0 = down, 2 = pending, 3 = maintenance
  time: string;
  ping: number | null;
  msg: string | null;
}

export interface MonitorData {
  kuma_id: number;
  heartbeats: HeartbeatEntry[];
}

type UptimeEvent =
  | { type: 'snapshot'; monitors: Record<number, MonitorData> }
  | { type: 'update'; kuma_id: number; entry: HeartbeatEntry };

// Module-level singleton state shared by all consumers
const monitors = ref<Map<number, MonitorData>>(new Map());
let eventSource: EventSource | null = null;
let consumerCount = 0;
const HEARTBEAT_WINDOW_SECS = 3600 * 2;

function openConnection() {
  if (eventSource) return;
  eventSource = new EventSource('/api/healthchecks/uptime/stream');

  eventSource.onmessage = (e: MessageEvent) => {
    try {
      const event: UptimeEvent = JSON.parse(e.data);
      if (event.type === 'snapshot') {
        const newMap = new Map<number, MonitorData>();
        for (const [id, monitor] of Object.entries(event.monitors)) {
          newMap.set(Number(id), monitor as MonitorData);
        }
        monitors.value = newMap;
      } else if (event.type === 'update') {
        const existing = monitors.value.get(event.kuma_id);
        const cutoff = Date.now() - HEARTBEAT_WINDOW_SECS * 1000;

        if (existing) {
          existing.heartbeats.push(event.entry);
          existing.heartbeats = existing.heartbeats.filter((h) => {
            return new Date(h.time).getTime() > cutoff;
          });
          // Trigger Vue reactivity for Map mutation
          monitors.value = new Map(monitors.value);
        } else {
          monitors.value.set(event.kuma_id, {
            kuma_id: event.kuma_id,
            heartbeats: [event.entry],
          });
          monitors.value = new Map(monitors.value);
        }
      }
    } catch {
      // Silently ignore malformed events
    }
  };

  eventSource.onerror = () => {
    // EventSource handles reconnect automatically.
    // On reconnect, the backend sends a fresh snapshot.
  };

  window.addEventListener('beforeunload', () => {
    if (eventSource) eventSource.close();
  });
}

function closeConnection() {
  if (eventSource) {
    eventSource.close();
    eventSource = null;
  }
}

export function useUptime() {
  consumerCount++;
  if (consumerCount === 1) {
    openConnection();
  }

  onUnmounted(() => {
    consumerCount--;
    if (consumerCount === 0) {
      closeConnection();
    }
  });

  function getMonitorData(kumaId: number): MonitorData | undefined {
    return monitors.value.get(kumaId);
  }

  return {
    monitors: monitors as Ref<Map<number, MonitorData>>,
    getMonitorData,
  };
}
