# 🚀 Phase 4 Quick Start Guide

**Target Session**: Next development session  
**Duration**: 12-16 hours  
**Objective**: Connect real API & WebSocket, replace all mock data  
**Status**: Ready to begin

---

## ⚡ TL;DR (5 Minutes)

### What You Need to Do
1. Replace mock metrics with real API calls
2. Connect WebSocket for real-time updates
3. Implement proxy control buttons
4. Add error handling & retry logic
5. Write unit tests

### Where to Start
**File**: `src/composables/dashboard/useDashboardMetrics.ts`  
**Line**: ~60 (the mock data section)  
**Change**: Replace `mockMetrics` with real `api.get('/api/dashboard/metrics')`

### Key Endpoints
```
GET  /api/dashboard/metrics           (1s poll)
GET  /api/dashboard/activity?limit=50 (on demand)
GET  /api/dashboard/connections       (on demand)
POST /api/proxy/start                 (action)
POST /api/proxy/stop                  (action)
DELETE /api/traffic                   (action)
```

### WebSocket Channels
```
ws://localhost:3000/ws?channel=metrics      (1s updates)
ws://localhost:3000/ws?channel=activity     (events)
ws://localhost:3000/ws?channel=connections  (events)
```

---

## 📋 Task Breakdown (Priority Order)

### Task 1: Replace Metrics API (2 hours)

**File**: `src/composables/dashboard/useDashboardMetrics.ts`

**Current Code** (lines ~60-75):
```typescript
// Mock data for development
const mockMetrics: SystemMetrics = {
  timestamp: Date.now(),
  requests_per_sec: Math.random() * 2000 + 100,
  // ... more mock fields
};

metrics.value = mockMetrics;
```

**New Code**:
```typescript
try {
  const response = await api.get<SystemMetrics>('/api/dashboard/metrics');
  metrics.value = response.data;
  lastUpdated.value = Date.now();
  retryCount = 0; // Reset on success
} catch (err) {
  // Handle error with retry logic
  const errorMsg = err instanceof Error ? err.message : 'Failed to fetch metrics';
  error.value = errorMsg;
  
  if (retryCount < MAX_RETRIES) {
    const delay = INITIAL_RETRY_DELAY * Math.pow(2, retryCount);
    retryCount++;
    retryTimeout = setTimeout(fetchMetrics, delay);
  }
}
```

**Testing**: 
- [ ] Run `npm run dev`
- [ ] Check Network tab (should see API call every 1 second)
- [ ] Verify metrics update in dashboard
- [ ] Check browser console (no errors)

---

### Task 2: Fetch Real Activity Data (1.5 hours)

**File**: `src/components/DashboardTab.vue`

**Current Code** (lines ~140-160):
```typescript
onMounted(() => {
  startAutoFetch(1000);
  
  // Mock activity data
  recentActivity.value = [
    { id: '1', timestamp: Date.now(), type: 'request', message: 'GET /api/users' },
    // ... more mock items
  ];
});
```

**New Code**:
```typescript
const fetchActivity = async () => {
  try {
    const response = await api.get<ActivityEntry[]>(
      '/api/dashboard/activity?limit=50'
    );
    recentActivity.value = response.data;
  } catch (err) {
    console.error('Failed to fetch activity:', err);
  }
};

onMounted(() => {
  startAutoFetch(1000);
  fetchActivity();
  
  // Fetch activity every 5 seconds
  setInterval(fetchActivity, 5000);
});
```

**API Response Expected**:
```typescript
{
  id: string;
  timestamp: number;
  type: 'request' | 'error' | 'warning' | 'info';
  message: string;
  details?: Record<string, any>;
  relatedId?: string;
}[]
```

---

### Task 3: Fetch Connection Stats (1 hour)

**File**: `src/components/dashboard/ConnectionGraph.vue`

**Current Code** (lines ~150-180):
```typescript
const connectionStats = ref<ConnectionStats>({
  active: 8,
  established: 6,
  // ... hardcoded mock values
});
```

**New Code**:
```typescript
const fetchConnections = async () => {
  try {
    const response = await api.get<ConnectionStats>(
      '/api/dashboard/connections'
    );
    connectionStats.value = response.data;
  } catch (err) {
    console.error('Failed to fetch connections:', err);
  }
};

onMounted(() => {
  fetchConnections();
  setInterval(fetchConnections, 5000); // Update every 5 seconds
});

onBeforeUnmount(() => {
  if (connectionInterval.value) {
    clearInterval(connectionInterval.value);
  }
});
```

---

### Task 4: Implement Proxy Control (1.5 hours)

**File**: `src/components/DashboardTab.vue`

**Current Code** (lines ~220-230):
```typescript
const toggleProxy = async () => {
  isUpdating.value = true;
  try {
    // TODO: Call API to toggle proxy
    proxyRunning.value = !proxyRunning.value;
  } finally {
    isUpdating.value = false;
  }
};
```

**New Code**:
```typescript
const toggleProxy = async () => {
  isUpdating.value = true;
  try {
    const endpoint = proxyRunning.value ? '/api/proxy/stop' : '/api/proxy/start';
    const response = await api.post<ProxyStatus>(endpoint);
    proxyRunning.value = response.data.running;
    
    // Show success notification
    showNotification(`Proxy ${response.data.running ? 'started' : 'stopped'}`, 'success');
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Failed to toggle proxy';
    showNotification(message, 'error');
  } finally {
    isUpdating.value = false;
  }
};

const clearTraffic = async () => {
  isUpdating.value = true;
  try {
    const response = await api.delete<{ cleared_count: number }>('/api/traffic');
    showNotification(`Cleared ${response.data.cleared_count} requests`, 'success');
  } catch (err) {
    showNotification('Failed to clear traffic', 'error');
  } finally {
    isUpdating.value = false;
  }
};

const exportMetrics = async () => {
  try {
    const response = await api.post('/api/dashboard/export?format=json');
    // Trigger file download
    const blob = new Blob([JSON.stringify(response.data)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `metrics-${Date.now()}.json`;
    a.click();
    showNotification('Metrics exported successfully', 'success');
  } catch (err) {
    showNotification('Failed to export metrics', 'error');
  }
};
```

---

### Task 5: Implement WebSocket (3 hours)

**File**: `src/composables/useWebSocket.ts`

**Create New File**:
```typescript
import { ref, computed } from 'vue';

interface WebSocketMessage<T> {
  type: string;
  data: T;
  timestamp: number;
}

export function useWebSocket<T>(
  channel: string,
  onMessage?: (data: T) => void
) {
  const ws = ref<WebSocket | null>(null);
  const isConnected = ref(false);
  const reconnectAttempts = ref(0);
  const maxReconnectAttempts = 5;
  
  const connect = () => {
    try {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const wsUrl = `${protocol}//${window.location.host}/ws?channel=${channel}`;
      
      ws.value = new WebSocket(wsUrl);
      
      ws.value.onopen = () => {
        isConnected.value = true;
        reconnectAttempts.value = 0;
        console.log(`WebSocket connected: ${channel}`);
      };
      
      ws.value.onmessage = (event: MessageEvent) => {
        try {
          const message = JSON.parse(event.data) as WebSocketMessage<T>;
          if (onMessage) {
            onMessage(message.data);
          }
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err);
        }
      };
      
      ws.value.onerror = (error: Event) => {
        console.error('WebSocket error:', error);
        isConnected.value = false;
      };
      
      ws.value.onclose = () => {
        isConnected.value = false;
        
        // Attempt to reconnect with exponential backoff
        if (reconnectAttempts.value < maxReconnectAttempts) {
          const delay = 1000 * Math.pow(2, reconnectAttempts.value);
          reconnectAttempts.value++;
          console.log(`Reconnecting in ${delay}ms (attempt ${reconnectAttempts.value})`);
          setTimeout(connect, delay);
        }
      };
    } catch (err) {
      console.error('Failed to establish WebSocket connection:', err);
      isConnected.value = false;
    }
  };
  
  const disconnect = () => {
    if (ws.value) {
      ws.value.close();
      ws.value = null;
    }
    isConnected.value = false;
  };
  
  return {
    isConnected: computed(() => isConnected.value),
    reconnectAttempts: computed(() => reconnectAttempts.value),
    connect,
    disconnect,
  };
}
```

**Usage in DashboardTab.vue**:
```typescript
import { useWebSocket } from '@/composables/useWebSocket';

const { isConnected: wsMetrics } = useWebSocket<SystemMetrics>(
  'metrics',
  (data) => {
    metrics.value = data;
    lastUpdated.value = Date.now();
  }
);

const { isConnected: wsActivity } = useWebSocket<ActivityEntry>(
  'activity',
  (data) => {
    recentActivity.value.unshift(data);
    if (recentActivity.value.length > 100) {
      recentActivity.value.pop(); // Keep max 100 items
    }
  }
);

onMounted(() => {
  // Try WebSocket first, fallback to polling
  wsMetrics.value ? startWebSocket() : startAutoFetch();
});
```

---

### Task 6: Add Error Handling & Notifications (1 hour)

**File**: `src/components/DashboardTab.vue`

**Add Notification System**:
```typescript
const notifications = ref<Array<{
  id: string;
  message: string;
  type: 'success' | 'error' | 'warning' | 'info';
  timestamp: number;
}>>([]);

const showNotification = (
  message: string,
  type: 'success' | 'error' | 'warning' | 'info' = 'info'
) => {
  const id = Math.random().toString(36);
  notifications.value.push({ id, message, type, timestamp: Date.now() });
  
  // Auto-remove after 5 seconds
  setTimeout(() => {
    notifications.value = notifications.value.filter(n => n.id !== id);
  }, 5000);
};

const dismissNotification = (id: string) => {
  notifications.value = notifications.value.filter(n => n.id !== id);
};
```

**Add Notification Component**:
```vue
<template>
  <div class="notifications">
    <div
      v-for="notif in notifications"
      :key="notif.id"
      class="notification"
      :class="notif.type"
    >
      <span>{{ notif.message }}</span>
      <button @click="dismissNotification(notif.id)">✕</button>
    </div>
  </div>
</template>
```

---

### Task 7: Write Unit Tests (3 hours)

**File**: `tests/unit/formatters.spec.ts`

```typescript
import { describe, it, expect } from 'vitest';
import { formatNumber, formatBytes, formatDuration, formatUptime } from '@/utils/dashboard/formatters';

describe('formatters', () => {
  describe('formatNumber', () => {
    it('should format 1234 as 1.2K', () => {
      expect(formatNumber(1234)).toBe('1.2K');
    });
    
    it('should format 1000000 as 1.0M', () => {
      expect(formatNumber(1000000)).toBe('1.0M');
    });
  });
  
  describe('formatBytes', () => {
    it('should format 1048576 as 1.0 MB', () => {
      expect(formatBytes(1048576)).toBe('1.0 MB');
    });
    
    it('should format 1073741824 as 1.0 GB', () => {
      expect(formatBytes(1073741824)).toBe('1.0 GB');
    });
  });
  
  describe('formatDuration', () => {
    it('should format 3000ms as 3.00s', () => {
      expect(formatDuration(3000)).toBe('3.00s');
    });
  });
  
  describe('formatUptime', () => {
    it('should format 90061 seconds as 1d 1m 1s', () => {
      expect(formatUptime(90061)).toBe('1d 1m 1s');
    });
  });
});
```

**File**: `tests/unit/thresholds.spec.ts`

```typescript
import { describe, it, expect } from 'vitest';
import { getHealthStatus } from '@/utils/dashboard/thresholds';

describe('thresholds', () => {
  describe('getHealthStatus', () => {
    it('should return healthy for values below warning', () => {
      expect(getHealthStatus(50, { warning: 70, critical: 85 })).toBe('healthy');
    });
    
    it('should return warning for values between warning and critical', () => {
      expect(getHealthStatus(75, { warning: 70, critical: 85 })).toBe('warning');
    });
    
    it('should return critical for values above critical', () => {
      expect(getHealthStatus(90, { warning: 70, critical: 85 })).toBe('critical');
    });
  });
});
```

---

## 🔧 Setup Checklist

Before starting development:

- [ ] **Backend is running**
  ```bash
  # Check if API is responding
  curl http://localhost:3000/api/dashboard/metrics
  ```

- [ ] **WebSocket server is running**
  ```bash
  # Check WebSocket endpoint
  wscat -c ws://localhost:3000/ws?channel=metrics
  ```

- [ ] **CORS is configured** (if needed)
  - Backend should allow requests from localhost:5173

- [ ] **Database has sample data**
  - At least 10+ requests captured
  - Activity log populated

- [ ] **Environment variables set** (if needed)
  ```bash
  VITE_API_URL=http://localhost:3000
  VITE_WS_URL=ws://localhost:3000
  ```

---

## 📚 File Structure Reference

```
src/
├── composables/
│   ├── useApi.ts ← API client helper
│   ├── useWebSocket.ts ← Create this for WebSocket
│   └── dashboard/
│       └── useDashboardMetrics.ts ← Modify (replace mock data)
│
├── components/
│   ├── DashboardTab.vue ← Modify (add API calls, WebSocket)
│   └── dashboard/
│       └── ConnectionGraph.vue ← Modify (add real connection data)
│
└── utils/
    └── dashboard/
        ├── formatters.ts ← Test these
        └── thresholds.ts ← Test these
```

---

## 🧪 Testing Workflow

### 1. Test Individual API Calls
```bash
# Open browser dev tools (F12)
# In Console tab:
fetch('http://localhost:3000/api/dashboard/metrics')
  .then(r => r.json())
  .then(d => console.log(d))
```

### 2. Test WebSocket
```bash
# In Console tab:
const ws = new WebSocket('ws://localhost:3000/ws?channel=metrics');
ws.onmessage = (e) => console.log(JSON.parse(e.data));
```

### 3. Test Dashboard Updates
```bash
npm run dev
# Open http://localhost:5173
# Navigate to Dashboard tab
# Watch Network tab for API requests
# Verify data updates every 1-2 seconds
```

### 4. Run Unit Tests
```bash
npm run test
```

---

## 🚨 Common Issues & Solutions

### Issue: "Failed to fetch from API"
**Solution**: 
- Check backend is running: `curl http://localhost:3000/api/dashboard/metrics`
- Check CORS headers in response
- Verify API URL in environment variables

### Issue: "WebSocket connection failed"
**Solution**:
- Check WebSocket server is running
- Verify WebSocket URL format: `ws://host:port/ws?channel=name`
- Check firewall/proxy settings

### Issue: "No data displaying"
**Solution**:
- Check Network tab in dev tools
- Verify API responses contain expected fields
- Check browser console for errors
- Verify TypeScript types match API response

### Issue: "Types don't match"
**Solution**:
- Ensure API response matches SystemMetrics interface
- Add missing fields to types/dashboard.ts
- Use `Partial<SystemMetrics>` if not all fields available

---

## ✅ Completion Checklist

### API Integration
- [ ] Metrics API working (real data, not mock)
- [ ] Activity API working (pagination)
- [ ] Connections API working (real stats)
- [ ] Proxy start/stop working
- [ ] Clear traffic working
- [ ] Export metrics working
- [ ] Error handling in place
- [ ] Retry logic implemented

### WebSocket Integration
- [ ] WebSocket composable created
- [ ] Metrics channel subscribed
- [ ] Activity channel subscribed
- [ ] Connections channel subscribed (optional)
- [ ] Reconnection logic working
- [ ] Status indicator showing connection state

### Testing
- [ ] Unit tests written (formatters, thresholds)
- [ ] Component tests written
- [ ] API integration tests
- [ ] Manual testing in browser
- [ ] All tests passing

### Cleanup
- [ ] Mock data removed
- [ ] Unused imports cleaned up
- [ ] Console.log statements removed
- [ ] TypeScript errors resolved
- [ ] ESLint warnings fixed

---

## 📞 Quick Commands

```bash
# Development
npm run dev                    # Start dev server
npm run type-check             # Check TypeScript
npm run lint                   # Check ESLint
npm run test                   # Run tests

# Build
npm run build                  # Production build
npm run preview               # Preview production build

# Testing
npm run test -- --watch       # Watch mode
npm run test -- --coverage    # Coverage report
```

---

## 🎯 Success Criteria for Phase 4

When Phase 4 is complete:

✅ All metrics are fetched from real API (not mock)  
✅ All activities are fetched from real API  
✅ Connection stats are fetched from real API  
✅ WebSocket provides real-time updates  
✅ Proxy can be started/stopped  
✅ Traffic can be cleared  
✅ Metrics can be exported  
✅ Error handling & retry logic work  
✅ Unit tests > 80% coverage  
✅ Zero TypeScript errors  
✅ Responsive design still works  
✅ No console errors  

---

## 📈 Time Estimate Breakdown

| Task | Duration | Status |
|------|----------|--------|
| 1. Replace Metrics API | 2h | ⏳ TODO |
| 2. Fetch Activity Data | 1.5h | ⏳ TODO |
| 3. Fetch Connections | 1h | ⏳ TODO |
| 4. Proxy Control | 1.5h | ⏳ TODO |
| 5. WebSocket | 3h | ⏳ TODO |
| 6. Error Handling | 1h | ⏳ TODO |
| 7. Unit Tests | 3h | ⏳ TODO |
| 8. Bug Fixes & Polish | 2h | ⏳ TODO |
| **Total** | **15h** | **Ready** |

---

## 🚀 You're Ready to Start!

Everything is prepared for Phase 4. The components are built, types are defined, and the architecture is solid. Now it's time to connect the real backend.

**Start with Task 1**: Replace mock metrics in useDashboardMetrics.ts

**Next**: Test the API call in the browser  

**Then**: Move to Task 2 and continue down the list  

**Finally**: Write tests and polish

Good luck! 🎉
