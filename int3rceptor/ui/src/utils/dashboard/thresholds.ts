/**
 * Dashboard Thresholds Utility
 *
 * Configuration and calculation of health status thresholds
 * Determines when metrics should trigger warning or critical alerts
 */

import type { HealthLevel, ThresholdConfig } from '@/types/dashboard';

/**
 * Threshold configuration for all dashboard metrics
 * Used to determine health status (healthy/warning/critical)
 */
export const THRESHOLDS = {
  cpu: {
    warning: 70,
    critical: 90,
  } as ThresholdConfig,

  memory: {
    warning: 80, // Percent of max
    critical: 95,
  } as ThresholdConfig,

  disk: {
    warning: 75,
    critical: 90,
  } as ThresholdConfig,

  responseTime: {
    warning: 500, // milliseconds
    critical: 1000,
  } as ThresholdConfig,

  errorRate: {
    warning: 1, // percent
    critical: 5,
  } as ThresholdConfig,

  connectionUtilization: {
    warning: 70, // percent of max connections
    critical: 90,
  } as ThresholdConfig,

  networkLatency: {
    warning: 100, // milliseconds
    critical: 300,
  } as ThresholdConfig,
};

/**
 * Determine health status based on current value and threshold config
 * Returns 'healthy', 'warning', or 'critical'
 *
 * @example
 * getHealthStatus(85, THRESHOLDS.cpu) → 'warning'
 * getHealthStatus(92, THRESHOLDS.cpu) → 'critical'
 * getHealthStatus(50, THRESHOLDS.cpu) → 'healthy'
 *
 * @param value - Current metric value
 * @param threshold - Threshold configuration
 * @returns Health status level
 */
export function getHealthStatus(
  value: number,
  threshold: ThresholdConfig
): HealthLevel {
  if (value >= threshold.critical) {
    return 'critical';
  }
  if (value >= threshold.warning) {
    return 'warning';
  }
  return 'healthy';
}

/**
 * Get CPU health status
 *
 * @param cpuPercent - CPU usage as percentage (0-100)
 * @returns Health status
 */
export function getCpuHealth(cpuPercent: number): HealthLevel {
  return getHealthStatus(cpuPercent, THRESHOLDS.cpu);
}

/**
 * Get memory health status
 *
 * @param memoryPercent - Memory usage as percentage (0-100)
 * @returns Health status
 */
export function getMemoryHealth(memoryPercent: number): HealthLevel {
  return getHealthStatus(memoryPercent, THRESHOLDS.memory);
}

/**
 * Get disk health status
 *
 * @param diskPercent - Disk usage as percentage (0-100)
 * @returns Health status
 */
export function getDiskHealth(diskPercent: number): HealthLevel {
  return getHealthStatus(diskPercent, THRESHOLDS.disk);
}

/**
 * Get response time health status
 *
 * @param responseTimeMs - Average response time in milliseconds
 * @returns Health status
 */
export function getResponseTimeHealth(responseTimeMs: number): HealthLevel {
  return getHealthStatus(responseTimeMs, THRESHOLDS.responseTime);
}

/**
 * Get error rate health status
 *
 * @param errorPercent - Error rate as percentage (0-100)
 * @returns Health status
 */
export function getErrorRateHealth(errorPercent: number): HealthLevel {
  return getHealthStatus(errorPercent, THRESHOLDS.errorRate);
}

/**
 * Get connection pool health status
 *
 * @param activeConnections - Number of active connections
 * @param maxConnections - Maximum connection limit
 * @returns Health status
 */
export function getConnectionHealth(
  activeConnections: number,
  maxConnections: number
): HealthLevel {
  const utilizationPercent = (activeConnections / maxConnections) * 100;
  return getHealthStatus(utilizationPercent, THRESHOLDS.connectionUtilization);
}

/**
 * Get network latency health status
 *
 * @param latencyMs - Network latency in milliseconds
 * @returns Health status
 */
export function getNetworkHealth(latencyMs: number): HealthLevel {
  return getHealthStatus(latencyMs, THRESHOLDS.networkLatency);
}

/**
 * Calculate overall health status from individual component statuses
 * Returns 'critical' if any component is critical
 * Returns 'warning' if any component is warning
 * Returns 'healthy' if all components are healthy
 *
 * @example
 * getOverallHealth(['healthy', 'warning', 'healthy']) → 'warning'
 * getOverallHealth(['healthy', 'critical', 'healthy']) → 'critical'
 * getOverallHealth(['healthy', 'healthy', 'healthy']) → 'healthy'
 *
 * @param statuses - Array of health statuses
 * @returns Overall health status
 */
export function getOverallHealth(statuses: HealthLevel[]): HealthLevel {
  if (statuses.some((s) => s === 'critical')) {
    return 'critical';
  }
  if (statuses.some((s) => s === 'warning')) {
    return 'warning';
  }
  return 'healthy';
}

/**
 * Check if value exceeds warning threshold
 *
 * @param value - Current value
 * @param threshold - Threshold configuration
 * @returns True if value >= warning threshold
 */
export function isWarning(value: number, threshold: ThresholdConfig): boolean {
  return value >= threshold.warning;
}

/**
 * Check if value exceeds critical threshold
 *
 * @param value - Current value
 * @param threshold - Threshold configuration
 * @returns True if value >= critical threshold
 */
export function isCritical(value: number, threshold: ThresholdConfig): boolean {
  return value >= threshold.critical;
}

/**
 * Get percentage to warning threshold
 * Returns how close a value is to the warning threshold
 * 0 = at minimum, 100 = at warning, 200+ = beyond warning
 *
 * @example
 * getWarningPercent(50, {warning: 70, critical: 90}) → 71.4
 * getWarningPercent(70, {warning: 70, critical: 90}) → 100
 * getWarningPercent(90, {warning: 70, critical: 90}) → 128.6
 *
 * @param value - Current value
 * @param threshold - Threshold configuration
 * @returns Percentage to warning threshold
 */
export function getWarningPercent(
  value: number,
  threshold: ThresholdConfig
): number {
  // Assume minimum is 0
  const range = threshold.warning;
  return (value / range) * 100;
}

/**
 * Get percentage to critical threshold
 * Returns how close a value is to the critical threshold
 *
 * @example
 * getCriticalPercent(50, {warning: 70, critical: 90}) → 55.6
 * getCriticalPercent(90, {warning: 70, critical: 90}) → 100
 *
 * @param value - Current value
 * @param threshold - Threshold configuration
 * @returns Percentage to critical threshold
 */
export function getCriticalPercent(
  value: number,
  threshold: ThresholdConfig
): number {
  const range = threshold.critical;
  return (value / range) * 100;
}

/**
 * Get remaining headroom before warning
 * Returns how much the value can increase before reaching warning
 *
 * @example
 * getWarningHeadroom(30, {warning: 70, critical: 90}) → 40
 * getWarningHeadroom(70, {warning: 70, critical: 90}) → 0
 * getWarningHeadroom(80, {warning: 70, critical: 90}) → -10
 *
 * @param value - Current value
 * @param threshold - Threshold configuration
 * @returns Remaining headroom (can be negative)
 */
export function getWarningHeadroom(
  value: number,
  threshold: ThresholdConfig
): number {
  return threshold.warning - value;
}

/**
 * Get remaining headroom before critical
 * Returns how much the value can increase before reaching critical
 *
 * @param value - Current value
 * @param threshold - Threshold configuration
 * @returns Remaining headroom (can be negative)
 */
export function getCriticalHeadroom(
  value: number,
  threshold: ThresholdConfig
): number {
  return threshold.critical - value;
}

/**
 * Predict when a metric will hit a threshold based on current trend
 * Simple linear extrapolation
 *
 * @example
 * // If memory grows 1MB per second and warning is at 80%
 * timeToThreshold(70, 1, 80) → 10 (seconds)
 *
 * @param currentValue - Current metric value
 * @param rateOfChange - Change per second
 * @param thresholdValue - Target threshold value
 * @returns Estimated seconds until threshold is reached
 */
export function timeToThreshold(
  currentValue: number,
  rateOfChange: number,
  thresholdValue: number
): number {
  if (rateOfChange === 0) {
    return Infinity;
  }

  const headroom = thresholdValue - currentValue;
  return Math.max(0, headroom / Math.abs(rateOfChange));
}

/**
 * Format threshold values for display
 *
 * @example
 * formatThreshold(THRESHOLDS.cpu) → "Warning: 70%, Critical: 90%"
 *
 * @param threshold - Threshold configuration
 * @param unit - Unit suffix (default: '%')
 * @returns Formatted threshold string
 */
export function formatThreshold(
  threshold: ThresholdConfig,
  unit: string = '%'
): string {
  return `Warning: ${threshold.warning}${unit}, Critical: ${threshold.critical}${unit}`;
}

/**
 * Get all thresholds for a metric
 * Returns a list of [label, value] pairs
 *
 * @param threshold - Threshold configuration
 * @returns Array of [label, value] pairs
 */
export function getThresholdValues(
  threshold: ThresholdConfig
): Array<[string, number]> {
  return [
    ['Warning', threshold.warning],
    ['Critical', threshold.critical],
  ];
}

/**
 * Validate threshold configuration
 * Ensures warning < critical
 *
 * @param threshold - Threshold configuration to validate
 * @returns True if threshold is valid
 */
export function isValidThreshold(threshold: ThresholdConfig): boolean {
  return threshold.warning < threshold.critical;
}

/**
 * Create custom threshold configuration
 * Validates and returns new threshold config
 *
 * @example
 * createThreshold(50, 80) → {warning: 50, critical: 80}
 *
 * @param warning - Warning threshold value
 * @param critical - Critical threshold value
 * @returns Threshold configuration
 * @throws Error if warning >= critical
 */
export function createThreshold(
  warning: number,
  critical: number
): ThresholdConfig {
  if (warning >= critical) {
    throw new Error('Warning threshold must be less than critical threshold');
  }

  return { warning, critical };
}

/**
 * Update a threshold configuration
 * Only updates provided fields
 *
 * @example
 * updateThreshold(THRESHOLDS.cpu, { warning: 75 })
 * → {warning: 75, critical: 90}
 *
 * @param threshold - Original threshold config
 * @param updates - Partial updates
 * @returns Updated threshold configuration
 */
export function updateThreshold(
  threshold: ThresholdConfig,
  updates: Partial<ThresholdConfig>
): ThresholdConfig {
  const updated = { ...threshold, ...updates };

  if (!isValidThreshold(updated)) {
    throw new Error('Updated threshold is invalid');
  }

  return updated;
}
