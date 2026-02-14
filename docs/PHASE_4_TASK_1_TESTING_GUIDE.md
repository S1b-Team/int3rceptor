# 🧪 Phase 4 Task 1: Practical Testing Guide

**Task**: Replace Metrics API with Real Backend Integration  
**Status**: ✅ Implementation Complete  
**Date**: 2025-01-20  
**Testing Scope**: All scenarios and integration points

---

## 🚀 Quick Start Testing (15 minutes)

### Prerequisites
- Backend running: `http://127.0.0.1:3000`
- Frontend dev server: `http://127.0.0.1:5173`
- DevTools open (F12)

### Step 1: Start Development Servers

**Terminal 1 - Backend API**:
```bash
cd int3rceptor/api
cargo run
# Expected: Listening on http://127.0.0.1:3000
```

**Terminal 2 - Frontend**:
```bash
cd int3rceptor/ui
npm run dev
# Expected: Local: http://127.0.0.1:5173
```

### Step 2: Open Dashboard in Browser
```
http://127.0.0.1:5173
Navigate to: Dashboard tab
```

### Step 3: Verify API is Connected
1. Open DevTools (F12)
2. Go to **Network** tab
3. Filter by: `/api/dashboard/metrics`
4. Should see requests every 1 second
5. Status: `200 OK`
6. Response contains: SystemMetrics object

### Step 4: Check Console Logs
1. DevTools → **Console** tab
2. Filter by: `[useDashboardMetrics]`
3. Look for: `Metrics fetched successfully`
4. No errors should be displayed

**Result**: ✅ API integration working!

---

## 📊 Testing Scenario 1: Successful API Calls

**Objective**: Verify metrics are fetched correctly and update in real-time

### Setup
1. Both servers running
2. Dashboard tab open
3. Network tab filtered to `/api/dashboard/metrics`

### Test Steps

1. **Observe Network Requests**
   - Watch Network tab
   - Every 1 second: new request appears
   - Status: 200 OK
   - Response size: ~400-500 bytes

2. **Check Metrics Display**
   - Metrics card shows: Requests/sec, Memory, etc.
   - Numbers change approximately every 1 second
   - No flickering or stalling

3. **Monitor Timestamps**
   - Check "Last Update" in header
   - Should change from "now" to "1s ago" to "2s ago"
   - Updates every 1 second

4. **Check Console Logs**
   - Filter by `[useDashboardMetrics]`
   - Should see: `Metrics fetched successfully`
   - Log every ~1 second
   - No error messages

### Expected Results

✅ Requests fire every 1 second  
✅ Status 200 OK  
✅ Response contains valid SystemMetrics  
✅ Metrics display updates  
✅ Timestamps progress correctly  
✅ Console shows success logs  
✅ No errors or warnings  

### Troubleshooting

**Problem**: No requests in Network tab  
**Solution**:
- Check backend is running: `curl http://127.0.0.1:3000/api/dashboard/metrics`
- Check frontend is at correct URL
- Refresh page (Ctrl+R)

**Problem**: 404 errors in Network tab  
**Solution**:
- Backend API endpoint not implemented
- Check `/api/dashboard/metrics` exists
- Verify backend running on port 3000

**Problem**: Metrics not updating  
**Solution**:
- Check `startAutoFetch()` was called
- Check for JavaScript errors in Console
- Verify `isLoading` becomes false

---

## ⚠️ Testing Scenario 2: Error Handling & Retry

**Objective**: Verify error handling and exponential backoff retry

### Setup
1. Both servers running
2. Dashboard open
3. Network tab open
4. Console visible (filter by `[useDashboardMetrics]`)

### Test Steps

#### Test 2A: Simulate Network Offline

1. **Enable Offline Mode**
   - DevTools → Network → Throttling
   - Change "No throttling" → "Offline"

2. **Observe Error Behavior**
   - Dashboard shows error notification
   - Error message: "Failed to fetch metrics"
   - Metrics card still visible (cached data)

3. **Monitor Retry Attempts**
   - Console shows: `Fetch failed`
   - Wait ~1 second
   - Console shows: `Retrying in 1024ms (attempt 1/3)`
   - Retry happens
   - Wait ~2 seconds
   - Second retry: `Retrying in 2048ms (attempt 2/3)`
   - Wait ~4 seconds
   - Third retry: `Retrying in 4096ms (attempt 3/3)`
   - After 3 failures: `Max retries exceeded`

4. **Re-enable Network**
   - DevTools → Network → Change back to "No throttling"
   - Wait for next retry or refresh

5. **Observe Recovery**
   - Metrics update again
   - Error notification disappears
   - Console: `Metrics fetched successfully`

### Expected Results

✅ Error message displays immediately  
✅ Retries occur at ~1s, ~2s, ~4s delays  
✅ Max 3 attempts enforced  
✅ Total retry wait: ~7 seconds  
✅ Error message shows after max retries  
✅ Metrics display doesn't break during error  
✅ Recovery works when network restored  
✅ Console shows all retry attempts  

### Metrics to Verify

```
First attempt: 0s (immediate)
Wait: ~1000ms (with jitter: 500-1500ms)
Second attempt: ~1000-1500ms
Wait: ~2000ms (with jitter: 1000-3000ms)
Third attempt: ~3000-4500ms
Wait: ~4000ms (with jitter: 2000-6000ms)
Fourth attempt: ~7000-10000ms
Result: Max retries exceeded error
```

---

## ⏱️ Testing Scenario 3: Request Timeout

**Objective**: Verify 5-second timeout is enforced

### Setup
1. Both servers running
2. Network throttling enabled (simulate slow connection)
3. DevTools open

### Test Steps

1. **Enable Slow 3G Throttling**
   - DevTools → Network → Throttling
   - Select "Slow 3G"

2. **Wait for Request to Timeout**
   - Watch Network tab
   - Request appears
   - Wait up to 5 seconds
   - Request should abort/timeout
   - Console shows: `Fetch failed` with timeout message

3. **Observe Retry Behavior**
   - Retry logic activates
   - Console shows retry attempts
   - Same exponential backoff as Scenario 2

4. **Restore Normal Throttling**
   - DevTools → Network → "No throttling"
   - Requests resume normally

### Expected Results

✅ Request times out after 5 seconds  
✅ Timeout triggers error state  
✅ Retry logic activates  
✅ Error message is user-friendly  
✅ Recovery works when throttling removed  
✅ No stalled/hanging requests  

---

## 🔄 Testing Scenario 4: Concurrent Request Prevention

**Objective**: Verify requests don't overlap even with fast polling

### Setup
1. Both servers running
2. Network tab open
3. DevTools Network throttling: "Slow 3G"

### Test Steps

1. **Monitor Request Timeline**
   - Watch Network tab carefully
   - Requests should be separated
   - Even if response time > polling interval
   - No overlapping/stacked requests

2. **Verify Timing**
   - Request starts at t=0s
   - Response time: 2-3 seconds (with Slow 3G)
   - Next request: after previous completes
   - NOT at t=1s (would overlap)

3. **Check Console**
   - No warnings about overlapping requests
   - One fetch at a time
   - `Metrics fetched successfully` once per request

### Expected Results

✅ No overlapping concurrent requests  
✅ Requests are sequential  
✅ Even with slow responses  
✅ Proper request ordering  
✅ No "double-fetch" issues  

---

## 🔧 Testing Scenario 5: Component Lifecycle

**Objective**: Verify proper cleanup on mount/unmount

### Setup
1. Both servers running
2. Dashboard tab open
3. Console visible

### Test Steps

#### Test 5A: Mount & Start Polling

1. Navigate to Dashboard tab
2. Check Console:
   - Should see: `Starting auto-fetch with 1000ms interval`
   - Metrics begin updating
   - Requests appear in Network tab

#### Test 5B: Navigate Away

1. Go to different tab (e.g., Requests tab)
2. Check Console:
   - Should see: `Auto-fetch stopped`
3. Check Network tab:
   - No more `/api/dashboard/metrics` requests
   - Previous requests complete but no new ones

#### Test 5C: Memory Check

1. Open DevTools → Memory tab
2. Take heap snapshot
3. Navigate back to Dashboard
4. Take another heap snapshot
5. Compare snapshots:
   - No significant memory increase
   - No dangling intervals
   - No memory leaks detected

#### Test 5D: Remount

1. Go back to Dashboard tab
2. Check Console:
   - Should see: `Starting auto-fetch again`
3. Check Network tab:
   - Requests resume immediately
   - Fresh polling cycle starts

### Expected Results

✅ Auto-fetch starts on mount  
✅ Auto-fetch stops on unmount  
✅ No memory leaks  
✅ No dangling intervals  
✅ Can remount cleanly  
✅ Fresh polling on remount  
✅ Proper lifecycle management  

---

## 🔐 Testing Scenario 6: Idempotency

**Objective**: Verify multiple start/stop calls are safe

### Setup
1. Backend running
2. Browser console open

### Test Steps

1. **Test Multiple startAutoFetch() Calls**
   ```javascript
   // In browser console:
   // Get composable reference (tricky, but possible via mounted component)
   // Try calling startAutoFetch() multiple times
   ```
   Expected: Warning message about already running, but no duplicate polling

2. **Test Multiple stopAutoFetch() Calls**
   ```javascript
   // In browser console:
   // Call stopAutoFetch() multiple times
   ```
   Expected: No errors, safe to call multiple times

3. **Test Rapid Mount/Unmount**
   - Rapidly navigate to Dashboard and away
   - Check Console for any errors
   - Check Network tab for orphaned requests
   Expected: No errors or memory leaks

### Expected Results

✅ Multiple `startAutoFetch()` safe  
✅ Multiple `stopAutoFetch()` safe  
✅ Rapid mount/unmount stable  
✅ No duplicate polling  
✅ No errors logged  
✅ No memory impact  

---

## 📈 State Management Testing

**Objective**: Verify reactive state updates correctly

### Browser Console Testing

```javascript
// Open DevTools Console
// Install Vue DevTools extension (if not already)
// In Vue DevTools, inspect DashboardTab component

// Check following refs update:
// ✓ metrics - changes with each fetch
// ✓ isLoading - true during fetch, false after
// ✓ error - null on success, message on error
// ✓ lastUpdated - timestamp updates each fetch
// ✓ timeSinceUpdate - computed value increments

// Manually test in console:
// const { metrics } = useDashboardMetrics();
// console.log(metrics.value); // Should see current metrics
```

### State Update Sequence (Normal Operation)

1. **Before Fetch**
   - `isLoading`: false
   - `error`: null
   - `metrics`: previous data (or null initially)

2. **During Fetch**
   - `isLoading`: true
   - `error`: null
   - `metrics`: unchanged (still previous data)

3. **After Successful Fetch**
   - `isLoading`: false
   - `error`: null
   - `metrics`: new data from API
   - `lastUpdated`: current timestamp

4. **On Error**
   - `isLoading`: false
   - `error`: error message
   - `metrics`: unchanged (cached)
   - `lastUpdated`: unchanged

### Expected Results

✅ All state refs update correctly  
✅ Loading state toggles properly  
✅ Error state sets/clears  
✅ Metrics update with new data  
✅ Timestamps increment  
✅ Computed properties reactive  

---

## 🐛 Debugging Checklist

### Console Logging

```javascript
// View all logs:
// Filter: [useDashboardMetrics]

// Expected logs:
// ✓ "Starting auto-fetch with 1000ms interval"
// ✓ "Metrics fetched successfully"
// ✓ "Fetch failed: ..."
// ✓ "Retrying in XXXms (attempt X/3)"
// ✓ "Auto-fetch stopped"
// ✓ "Max retries exceeded"
```

### Network Tab Monitoring

```
Filter: /api/dashboard/metrics

✓ Method: GET
✓ Status: 200 OK (or error status on failure)
✓ Response type: json
✓ Size: ~400-500 bytes
✓ Time: <100ms (typically)
✓ Frequency: every 1 second
```

### Vue DevTools Inspection

```
Component: DashboardTab
Props: (none relevant)
Emits: (none)
Exposed: (check metrics composable values)

Composable State (useDashboardMetrics):
  ✓ metrics: SystemMetrics object
  ✓ isLoading: boolean
  ✓ error: string | null
  ✓ lastUpdated: number (timestamp)
  ✓ timeSinceUpdate: computed number
```

---

## 📋 Complete Testing Checklist

### Before Testing
- [ ] Backend running on port 3000
- [ ] Frontend running on port 5173
- [ ] DevTools installed
- [ ] Network tab available
- [ ] Console visible

### Scenario 1: Successful Calls
- [ ] Requests appear every 1 second
- [ ] Status 200 OK
- [ ] Metrics display updates
- [ ] Last update timestamp progresses
- [ ] Console shows success logs
- [ ] No errors

### Scenario 2: Error & Retry
- [ ] Error notification appears
- [ ] Retries at 1s, 2s, 4s
- [ ] Console shows retry logs
- [ ] Max 3 attempts enforced
- [ ] Recovery works
- [ ] Error dismisses

### Scenario 3: Timeout
- [ ] 5-second timeout enforced
- [ ] Timeout triggers error
- [ ] Retry activates
- [ ] Message is user-friendly

### Scenario 4: Concurrent Prevention
- [ ] No overlapping requests
- [ ] Sequential execution
- [ ] Proper ordering

### Scenario 5: Lifecycle
- [ ] Auto-fetch starts on mount
- [ ] Auto-fetch stops on unmount
- [ ] No memory leaks
- [ ] Can remount cleanly

### Scenario 6: Idempotency
- [ ] Safe to call multiple times
- [ ] No duplicate polling
- [ ] No errors

### State Management
- [ ] All refs update correctly
- [ ] Loading toggles
- [ ] Error sets/clears
- [ ] Metrics update
- [ ] Timestamps progress

---

## 🎯 Success Criteria - All Must Pass

| Test | Status | Notes |
|------|--------|-------|
| API requests fire every 1s | ✅/❌ | Check Network tab |
| Status 200 OK | ✅/❌ | No 4xx or 5xx |
| Metrics display updates | ✅/❌ | Visible changes |
| Error handling works | ✅/❌ | Offline mode test |
| Retry logic works | ✅/❌ | 1s, 2s, 4s delays |
| Timeout enforced | ✅/❌ | 5 seconds max |
| No concurrent requests | ✅/❌ | Slow 3G test |
| Cleanup on unmount | ✅/❌ | Network stops |
| No memory leaks | ✅/❌ | Heap snapshot |
| Can remount | ✅/❌ | Polling restarts |
| Idempotency safe | ✅/❌ | No double polling |
| State updates | ✅/❌ | Vue DevTools |
| No console errors | ✅/❌ | Filter by component |
| No console warnings | ✅/❌ | Check all warnings |

---

## 🚀 Quick Test Commands

### Terminal Commands

```bash
# Verify backend API exists
curl http://127.0.0.1:3000/api/dashboard/metrics

# Expected response:
{
  "timestamp": 1705769400000,
  "requests_per_sec": 1234.56,
  "avg_response_time_ms": 45.23,
  "memory_usage_mb": 128.45,
  ...
}

# Check CORS headers
curl -I http://127.0.0.1:3000/api/dashboard/metrics

# Expected headers:
# Access-Control-Allow-Origin: *
# Content-Type: application/json
```

### Browser Console Commands

```javascript
// Check if composable is accessible (if exposed in component)
// This depends on your component setup

// Check Network tab filtering
// DevTools → Network → Filter input → Type: /api/dashboard/metrics

// Monitor logs
// DevTools → Console → Filter input → Type: [useDashboardMetrics]

// Check Vue component state
// DevTools → Vue → Components → Find DashboardTab → Inspect
```

---

## 📊 Performance Metrics to Monitor

### Network Performance

```
Request Frequency: 1 per second
Response Size: ~400-500 bytes
Response Time: <100ms typical
Bandwidth: ~40-50 KB per minute
Daily Data: ~43 MB per client
```

### Memory Performance

```
Composable Instance: <2 KB
State Refs: ~1 KB
Intervals: None active (cleared)
Total Per Instance: <3 KB
```

### CPU Performance

```
JSON Parsing: <1ms per request
Vue Reactivity: <2ms per update
Total Per Second: <5ms
CPU Impact: Negligible (<0.5%)
```

---

## 🔗 Related Documentation

- **Implementation**: PHASE_4_TASK_1_IMPLEMENTATION.md
- **Integration**: PHASE_4_TASK_1_INTEGRATION.md
- **Summary**: PHASE_4_TASK_1_SUMMARY.md
- **Completion**: PHASE_4_COMPLETION_REPORT.md

---

## 📞 Troubleshooting Guide

### Issue: "Failed to fetch metrics"

**Possible Causes**:
1. Backend not running
2. API endpoint not implemented
3. CORS misconfiguration
4. Network connectivity

**Solutions**:
```bash
# 1. Check backend
curl http://127.0.0.1:3000/api/dashboard/metrics

# 2. Check CORS headers
curl -I http://127.0.0.1:3000/api/dashboard/metrics

# 3. Check console for errors
# DevTools → Console → Look for CORS errors

# 4. Check network details
# DevTools → Network → Click request → Response tab
```

### Issue: "WebSocket connection failed"

**Note**: This is Phase 5, not implemented yet. Task 1 only uses HTTP polling.

### Issue: "Memory leak detected"

**Possible Causes**:
1. `stopAutoFetch()` not called
2. Interval not cleared
3. Mounted hook issue

**Solutions**:
```javascript
// Verify lifecycle:
// onMounted(() => startAutoFetch());
// onBeforeUnmount(() => stopAutoFetch()); // Important!

// Check memory manually:
// DevTools → Memory → Heap Snapshot → Compare
```

### Issue: Duplicate polling

**Possible Causes**:
1. Multiple `startAutoFetch()` calls
2. Component remounted without unmount

**Solutions**:
```javascript
// Check composable idempotency:
// startAutoFetch() should warn if already running
// Look in console for: "Auto-fetch already running"

// Verify no duplicate components:
// DevTools → Vue → Components → Check DashboardTab count
```

---

## ✅ Testing Sign-Off

After passing all scenarios:

```
✅ API Integration Working
✅ Error Handling Complete
✅ Retry Logic Correct
✅ Timeout Protection Active
✅ Lifecycle Management Clean
✅ No Memory Leaks
✅ No Console Errors
✅ Production Ready
```

---

**Last Updated**: 2025-01-20  
**Status**: Ready for Testing  
**Estimated Duration**: 1-2 hours (all scenarios)  
**Difficulty**: Easy (mostly UI/Network observation)
