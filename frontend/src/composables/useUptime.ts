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
let closeTimer: ReturnType<typeof setTimeout> | null = null;
const HEARTBEAT_WINDOW_SIZE = 120;
// Grace period before tearing down the stream once the last consumer unmounts.
// Bridges route navigations (old component unmounts before the new one mounts)
// so the long-lived connection survives instead of reconnecting every page hop.
const CLOSE_GRACE_MS = 5000;

// Registered once for the app lifetime, not per-connection.
window.addEventListener('beforeunload', () => {
  if (eventSource) eventSource.close();
});

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

        if (existing) {
          existing.heartbeats.push(event.entry);
          existing.heartbeats.splice(
            0,
            Math.max(0, existing.heartbeats.length - HEARTBEAT_WINDOW_SIZE)
          );
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
}

function closeConnection() {
  if (eventSource) {
    eventSource.close();
    eventSource = null;
  }
}

export function useUptime() {
  // A new consumer arrived — cancel any pending teardown so navigation
  // between uptime views reuses the existing connection.
  if (closeTimer !== null) {
    clearTimeout(closeTimer);
    closeTimer = null;
  }
  consumerCount++;
  openConnection();

  onUnmounted(() => {
    consumerCount--;
    if (consumerCount === 0 && closeTimer === null) {
      closeTimer = setTimeout(() => {
        closeTimer = null;
        if (consumerCount === 0) closeConnection();
      }, CLOSE_GRACE_MS);
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
