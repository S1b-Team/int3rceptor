// ==========================================
// DASHBOARD - Real-time Metrics & Monitoring
// ==========================================

/**
 * SystemMetrics - Real-time system performance metrics
 * Captured every 1 second via polling or WebSocket
 */
export interface SystemMetrics {
    timestamp: number;
    requests_per_sec: number;
    avg_response_time_ms: number;
    memory_usage_mb: number;
    active_connections: number;
    bytes_in_sec: number;
    bytes_out_sec: number;
    error_rate_percent: number;
    cpu_percent?: number;
    disk_percent?: number;
    uptime_seconds: number;
}

/**
 * MetricPoint - Single data point in a time series
 * Used for sparklines and historical chart data
 */
export interface MetricPoint {
    timestamp: number;
    value: number;
    label?: string;
}

/**
 * TimeSeriesData - Complete time series for a single metric
 * Includes historical data, statistics, and percentiles
 */
export interface TimeSeriesData {
    metric: string;
    unit: string;
    dataPoints: MetricPoint[];
    current: number;
    min: number;
    max: number;
    avg: number;
    percentile_95: number;
    percentile_99: number;
}

/**
 * MetricCardProps - Props for the MetricCard component
 */
export interface MetricCardProps {
    title: string;
    value: number;
    unit: string;
    icon?: string;
    trend?: {
        direction: "up" | "down" | "stable";
        percent: number;
        timeWindow?: string;
    };
    threshold?: {
        warning: number;
        critical: number;
        max: number;
    };
    sparklineData?: MetricPoint[];
    loading?: boolean;
}

/**
 * HealthLevel - Severity level for system health
 */
export type HealthLevel = "healthy" | "warning" | "critical";

/**
 * HealthStatus - Overall system health with component-level statuses
 */
export interface HealthStatus {
    overall: HealthLevel;
    cpu: HealthLevel;
    memory: HealthLevel;
    disk: HealthLevel;
    network: HealthLevel;
    database: HealthLevel;
    uptime: number;
    lastCheck: number;
}

/**
 * HealthGaugeData - Data for a single health gauge component
 */
export interface HealthGaugeData {
    label: string;
    current: number;
    max: number;
    unit: string;
    status: HealthLevel;
    trend?: number;
}

/**
 * ActivityEntry - Single event in activity log
 * Tracks important events like requests, errors, warnings
 */
export interface ActivityEntry {
    id: string;
    timestamp: number;
    type: "request" | "error" | "warning" | "info";
    message: string;
    details?: Record<string, any>;
    relatedId?: string;
}

/**
 * TopEndpoint - Most frequently accessed endpoint statistics
 */
export interface TopEndpoint {
    url: string;
    host: string;
    requests_count: number;
    total_bytes: number;
    avg_response_time: number;
    error_count: number;
    last_seen: number;
}

/**
 * ConnectionStats - Connection pool statistics
 */
export interface ConnectionStats {
    active: number;
    established: number;
    closing: number;
    failed: number;
    total_lifetime: number;
    avg_duration_ms: number;
    peak_connections: number;
    peak_time: number;
    concurrent_limit: number;
}

/**
 * ConnectionBreakdown - Connection statistics by type and state
 */
export interface ConnectionBreakdown {
    protocol: {
        http: number;
        https: number;
        websocket: number;
    };
    state: {
        connecting: number;
        open: number;
        closing: number;
        closed: number;
    };
}

/**
 * ProxyStatus - Current proxy server status and configuration
 */
export interface ProxyStatus {
    running: boolean;
    host: string;
    port: number;
    tls_enabled: boolean;
    intercept_https: boolean;
    start_time: number;
    certificates_generated: number;
}

/**
 * ProxyConfig - Configuration for proxy server
 */
export interface ProxyConfig {
    port: number;
    host: string;
    tls_enabled: boolean;
    intercept_https: boolean;
    max_connections: number;
    request_timeout_ms: number;
    idle_timeout_ms: number;
}

/**
 * DashboardState - Complete state shape for the dashboard
 * Includes all metrics, status, and UI state
 */
export interface DashboardState {
    metrics: SystemMetrics | null;
    timeSeries: Map<string, TimeSeriesData>;
    connectionStats: ConnectionStats;
    recentActivity: ActivityEntry[];
    topEndpoints: TopEndpoint[];
    systemHealth: HealthStatus;
    proxyStatus: ProxyStatus;
    lastUpdated: number;
    isLoading: boolean;
    error: string | null;
}

/**
 * MetricUpdate - Incremental metric update for batching
 * Used to batch multiple updates before applying to state
 */
export interface MetricsUpdate {
    id: string;
    timestamp: number;
    changes: Partial<SystemMetrics>;
    source: "polling" | "websocket" | "manual";
}

/**
 * DashboardEvent - WebSocket event from server
 * Tagged union type for type-safe event handling
 */
export type DashboardEvent =
    | { type: "metrics_update"; payload: SystemMetrics }
    | { type: "connection_change"; payload: ConnectionStats }
    | { type: "activity_log"; payload: ActivityEntry }
    | { type: "health_change"; payload: HealthStatus }
    | { type: "proxy_status"; payload: ProxyStatus }
    | { type: "error"; payload: { code: string; message: string } };

/**
 * ChartDataPoint - Single point in a chart (for TimeSeriesChart)
 */
export interface ChartDataPoint {
    timestamp: number;
    value: number;
    series?: string;
}

/**
 * ThresholdConfig - Configuration for health status thresholds
 */
export interface ThresholdConfig {
    warning: number;
    critical: number;
}

/**
 * FormatOptions - Options for formatting numbers/values
 */
export interface FormatOptions {
    decimals?: number;
    unit?: string;
    suffix?: string;
}

/**
 * DashboardComposableReturn - Return type for useDashboardMetrics
 */
export interface UseDashboardMetricsReturn {
    metrics: SystemMetrics | null;
    isLoading: boolean;
    error: string | null;
    lastUpdated: number;
    fetchMetrics: () => Promise<void>;
    startAutoFetch: (intervalMs?: number) => void;
    stopAutoFetch: () => void;
}

/**
 * DashboardWebSocketReturn - Return type for useDashboardWebSocket
 */
export interface UseDashboardWebSocketReturn {
    isConnected: boolean;
    lastEvent: DashboardEvent | null;
    reconnectAttempts: number;
    connect: () => void;
    disconnect: () => void;
}

/**
 * SystemHealthReturn - Return type for useSystemHealth
 */
export interface UseSystemHealthReturn {
    health: HealthStatus;
    getHealthLevel: (value: number, threshold: ThresholdConfig) => HealthLevel;
}

/**
 * DashboardExportData - Data structure for exporting dashboard metrics
 */
export interface DashboardExportData {
    exportedAt: string;
    format: "json" | "csv" | "har";
    metrics: SystemMetrics[];
    healthHistory: HealthStatus[];
    activityLog: ActivityEntry[];
    summary: {
        totalRequests: number;
        averageResponseTime: number;
        peakMemoryUsage: number;
        totalErrors: number;
    };
}

/**
 * PaginationOptions - Options for paginated API requests
 */
export interface PaginationOptions {
    limit?: number;
    offset?: number;
}

/**
 * ApiResponse - Generic API response wrapper
 */
export interface ApiResponse<T> {
    success: boolean;
    data?: T;
    error?: {
        code: string;
        message: string;
    };
    timestamp: number;
}
