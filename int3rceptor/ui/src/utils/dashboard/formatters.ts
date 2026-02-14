/**
 * Dashboard Formatters Utility
 *
 * Formatting functions for displaying metrics, numbers, times, and data sizes
 * Used throughout the dashboard to present data in human-readable formats
 */

/**
 * Format a large number with appropriate unit suffix (K, M, B, T)
 *
 * @example
 * formatNumber(1234) → "1.2K"
 * formatNumber(1234567) → "1.2M"
 * formatNumber(1234567890) → "1.2B"
 *
 * @param value - The number to format
 * @param decimals - Number of decimal places (default: 1)
 * @param suffix - Optional suffix to append (e.g., "/sec")
 * @returns Formatted string with unit
 */
export function formatNumber(
  value: number,
  decimals: number = 1,
  suffix: string = ''
): string {
  if (value === 0) return '0' + suffix;

  const units = ['', 'K', 'M', 'B', 'T'];
  const k = 1000;
  const exponent = Math.floor(Math.log10(Math.abs(value)) / Math.log10(k));

  if (exponent === 0) {
    return Math.round(value).toString() + suffix;
  }

  const scaled = value / Math.pow(k, exponent);
  return scaled.toFixed(decimals) + units[exponent] + suffix;
}

/**
 * Format bytes as human-readable file size (B, KB, MB, GB, TB)
 *
 * @example
 * formatBytes(1024) → "1.0 KB"
 * formatBytes(1048576) → "1.0 MB"
 * formatBytes(1234567890) → "1.1 GB"
 *
 * @param bytes - Number of bytes
 * @param decimals - Number of decimal places (default: 1)
 * @returns Formatted string with unit
 */
export function formatBytes(bytes: number, decimals: number = 1): string {
  if (bytes === 0) return '0 B';

  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const k = 1024;
  const exponent = Math.floor(Math.log(bytes) / Math.log(k));

  const value = bytes / Math.pow(k, exponent);
  return value.toFixed(decimals) + ' ' + units[exponent];
}

/**
 * Format milliseconds as human-readable duration
 *
 * @example
 * formatDuration(500) → "500ms"
 * formatDuration(1500) → "1.50s"
 * formatDuration(65000) → "1.08m"
 * formatDuration(3665000) → "1.02h"
 *
 * @param milliseconds - Duration in milliseconds
 * @returns Formatted duration string
 */
export function formatDuration(milliseconds: number): string {
  if (milliseconds < 1000) {
    return `${Math.round(milliseconds)}ms`;
  }

  if (milliseconds < 60_000) {
    return `${(milliseconds / 1000).toFixed(2)}s`;
  }

  if (milliseconds < 3_600_000) {
    return `${(milliseconds / 60_000).toFixed(2)}m`;
  }

  return `${(milliseconds / 3_600_000).toFixed(2)}h`;
}

/**
 * Format seconds as human-readable uptime
 *
 * @example
 * formatUptime(61) → "1m 1s"
 * formatUptime(3661) → "1h 1m 1s"
 * formatUptime(86461) → "1d 1m 1s"
 *
 * @param seconds - Uptime in seconds
 * @returns Formatted uptime string
 */
export function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  const parts = [];
  if (days > 0) parts.push(`${days}d`);
  if (hours > 0) parts.push(`${hours}h`);
  if (minutes > 0) parts.push(`${minutes}m`);
  if (secs > 0 || parts.length === 0) parts.push(`${secs}s`);

  return parts.join(' ');
}

/**
 * Format percentage with appropriate decimal places
 *
 * @example
 * formatPercent(12.456) → "12.5%"
 * formatPercent(5) → "5.0%"
 * formatPercent(0.123) → "0.1%"
 *
 * @param value - Percentage value
 * @param decimals - Number of decimal places (default: 1)
 * @returns Formatted percentage string
 */
export function formatPercent(value: number, decimals: number = 1): string {
  return `${value.toFixed(decimals)}%`;
}

/**
 * Format timestamp as readable time (HH:MM:SS)
 *
 * @example
 * formatTime(Date.now()) → "14:30:45"
 * formatTime(1234567890000) → "01:31:30"
 *
 * @param timestamp - Timestamp in milliseconds
 * @returns Formatted time string
 */
export function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('en-US', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
}

/**
 * Format timestamp as readable date and time
 *
 * @example
 * formatDateTime(Date.now()) → "Jan 20, 2025 14:30:45"
 *
 * @param timestamp - Timestamp in milliseconds
 * @returns Formatted date and time string
 */
export function formatDateTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
}

/**
 * Format URL with truncation for display
 *
 * @example
 * formatUrl("https://api.example.com/users/123/profile", 50)
 * → "https://api.example.com/users/123/pro..."
 *
 * @param url - Full URL string
 * @param maxLength - Maximum length before truncation (default: 60)
 * @returns Formatted URL with ellipsis if truncated
 */
export function formatUrl(url: string, maxLength: number = 60): string {
  if (url.length <= maxLength) return url;
  return url.substring(0, maxLength - 3) + '...';
}

/**
 * Get CSS class name based on health status
 *
 * @example
 * getHealthClass('healthy') → 'status-healthy'
 * getHealthClass('warning') → 'status-warning'
 * getHealthClass('critical') → 'status-critical'
 *
 * @param level - Health level ('healthy' | 'warning' | 'critical')
 * @returns CSS class name
 */
export function getHealthClass(
  level: 'healthy' | 'warning' | 'critical'
): string {
  return `status-${level}`;
}

/**
 * Get color hex code based on health status
 *
 * @example
 * getHealthColor('healthy') → '#00d4ff'
 * getHealthColor('warning') → '#ffb800'
 * getHealthColor('critical') → '#ff006e'
 *
 * @param level - Health level ('healthy' | 'warning' | 'critical')
 * @returns Hex color code
 */
export function getHealthColor(
  level: 'healthy' | 'warning' | 'critical'
): string {
  const colors = {
    healthy: '#00d4ff',    // Cyan
    warning: '#ffb800',    // Orange
    critical: '#ff006e',   // Magenta
  };
  return colors[level];
}

/**
 * Get status badge label based on HTTP status code
 *
 * @example
 * getStatusLabel(200) → '200 OK'
 * getStatusLabel(404) → '404 Not Found'
 * getStatusLabel(500) → '500 Error'
 *
 * @param statusCode - HTTP status code
 * @returns Formatted status label
 */
export function getStatusLabel(statusCode: number): string {
  const labels: Record<number, string> = {
    200: 'OK',
    201: 'Created',
    204: 'No Content',
    301: 'Moved Permanently',
    302: 'Found',
    304: 'Not Modified',
    400: 'Bad Request',
    401: 'Unauthorized',
    403: 'Forbidden',
    404: 'Not Found',
    429: 'Too Many Requests',
    500: 'Internal Server Error',
    502: 'Bad Gateway',
    503: 'Service Unavailable',
  };

  const label = labels[statusCode] || 'Unknown';
  return `${statusCode} ${label}`;
}

/**
 * Get CSS class for HTTP method badge
 *
 * @example
 * getMethodClass('GET') → 'method-get'
 * getMethodClass('POST') → 'method-post'
 * getMethodClass('DELETE') → 'method-delete'
 *
 * @param method - HTTP method
 * @returns CSS class name
 */
export function getMethodClass(method: string): string {
  return `method-${method.toLowerCase()}`;
}

/**
 * Get color for HTTP method badge
 *
 * @example
 * getMethodColor('GET') → '#00d4ff'
 * getMethodColor('POST') → '#ff006e'
 * getMethodColor('DELETE') → '#ff0000'
 *
 * @param method - HTTP method
 * @returns Hex color code
 */
export function getMethodColor(method: string): string {
  const colors: Record<string, string> = {
    GET: '#00d4ff',      // Cyan
    POST: '#ff006e',     // Magenta
    PUT: '#ffb800',      // Orange
    DELETE: '#ff0000',   // Red
    PATCH: '#8b5cf6',    // Purple
    HEAD: '#a0a0a0',     // Gray
    OPTIONS: '#a0a0a0',  // Gray
  };

  return colors[method.toUpperCase()] || '#ffffff';
}

/**
 * Format metric value with appropriate scaling
 * Automatically chooses between integers, decimals, or scientific notation
 *
 * @example
 * formatMetricValue(1234.567) → "1234.6"
 * formatMetricValue(1000000) → "1.0M"
 * formatMetricValue(0.00123) → "0.0012"
 *
 * @param value - Numeric value
 * @param decimals - Decimal places (default: 1)
 * @returns Formatted metric value
 */
export function formatMetricValue(value: number, decimals: number = 1): string {
  // For very small numbers, use scientific notation
  if (Math.abs(value) < 0.001 && value !== 0) {
    return value.toExponential(decimals);
  }

  // For large numbers, use K/M/B notation
  if (Math.abs(value) >= 1000) {
    return formatNumber(value, decimals);
  }

  // For normal numbers, use fixed decimals
  return value.toFixed(decimals);
}

/**
 * Capitalize first letter of string
 *
 * @example
 * capitalize('hello') → 'Hello'
 * capitalize('connection') → 'Connection'
 *
 * @param str - String to capitalize
 * @returns Capitalized string
 */
export function capitalize(str: string): string {
  if (!str) return '';
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Convert camelCase to Title Case
 *
 * @example
 * camelToTitle('activeConnections') → 'Active Connections'
 * camelToTitle('memoryUsageMb') → 'Memory Usage Mb'
 *
 * @param str - Camel case string
 * @returns Title case string
 */
export function camelToTitle(str: string): string {
  return str
    .replace(/([A-Z])/g, ' $1')
    .replace(/^./, (char) => char.toUpperCase())
    .trim();
}

/**
 * Round number to specified decimal places
 *
 * @example
 * roundTo(1.2345, 2) → 1.23
 * roundTo(12345, -2) → 12300
 *
 * @param value - Number to round
 * @param decimals - Number of decimal places (negative for rounding left)
 * @returns Rounded number
 */
export function roundTo(value: number, decimals: number = 0): number {
  const factor = Math.pow(10, decimals);
  return Math.round(value * factor) / factor;
}

/**
 * Clamp number between min and max
 *
 * @example
 * clamp(15, 0, 10) → 10
 * clamp(-5, 0, 10) → 0
 * clamp(5, 0, 10) → 5
 *
 * @param value - Value to clamp
 * @param min - Minimum value
 * @param max - Maximum value
 * @returns Clamped value
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}
