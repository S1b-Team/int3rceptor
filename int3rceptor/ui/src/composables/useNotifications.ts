/**
 * useNotifications Composable
 *
 * Manages application notifications/toasts with automatic dismissal.
 * - Toast notifications with type (success, error, warning, info)
 * - Auto-dismiss after configurable duration
 * - Manual dismiss functionality
 * - Reactive notification queue
 *
 * @example
 * const { notifications, showNotification, dismissNotification } = useNotifications();
 * showNotification('Operation successful', 'success');
 */

import { ref, computed, type Ref, type ComputedRef } from "vue";

// ==========================================
// Configuration Constants
// ==========================================

/** Default auto-dismiss duration in milliseconds */
const DEFAULT_DURATION_MS = 5000;

/** Duration for error notifications in milliseconds */
const ERROR_DURATION_MS = 7000;

/** Duration for success notifications in milliseconds */
const SUCCESS_DURATION_MS = 4000;

/** Maximum number of notifications to keep */
const MAX_NOTIFICATIONS = 10;

// ==========================================
// Type Definitions
// ==========================================

/**
 * Notification type
 */
export type NotificationType = "success" | "error" | "warning" | "info";

/**
 * Single notification object
 */
export interface Notification {
    id: string;
    message: string;
    type: NotificationType;
    timestamp: number;
    duration?: number;
    dismissible?: boolean;
}

/**
 * Return type for the useNotifications composable
 */
interface UseNotificationsReturn {
    notifications: Ref<Notification[]>;
    notificationCount: ComputedRef<number>;
    hasErrors: ComputedRef<boolean>;
    showNotification: (
        message: string,
        type?: NotificationType,
        duration?: number,
    ) => string;
    showSuccess: (message: string, duration?: number) => string;
    showError: (message: string, duration?: number) => string;
    showWarning: (message: string, duration?: number) => string;
    showInfo: (message: string, duration?: number) => string;
    dismissNotification: (id: string) => void;
    dismissAll: () => void;
    dismissByType: (type: NotificationType) => void;
}

// ==========================================
// Composable Implementation
// ==========================================

/**
 * Composable for managing toast notifications
 *
 * Handles:
 * - Creating notifications with different types
 * - Auto-dismissing after configurable duration
 * - Manual dismissal
 * - Notification queue management
 * - Memory leak prevention
 *
 * @returns {UseNotificationsReturn} Notification state and control functions
 */
export function useNotifications(): UseNotificationsReturn {
    // ==========================================
    // State
    // ==========================================

    /** List of active notifications */
    const notifications = ref<Notification[]>([]);

    // ==========================================
    // Internal Tracking
    // ==========================================

    /** Map of notification ID to dismiss timeout */
    const dismissTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

    // ==========================================
    // Computed Properties
    // ==========================================

    /**
     * Computed: Total number of active notifications
     */
    const notificationCount = computed((): number => {
        return notifications.value.length;
    });

    /**
     * Computed: Whether there are any error notifications
     */
    const hasErrors = computed((): boolean => {
        return notifications.value.some((n) => n.type === "error");
    });

    // ==========================================
    // Helper Functions
    // ==========================================

    /**
     * Generate unique notification ID
     *
     * @returns Unique ID string
     */
    const generateId = (): string => {
        return `notif_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    };

    /**
     * Get duration for notification type
     *
     * @param type - Notification type
     * @param customDuration - Optional custom duration override
     * @returns Duration in milliseconds
     */
    const getDuration = (
        type: NotificationType,
        customDuration?: number,
    ): number => {
        if (customDuration !== undefined) {
            return customDuration;
        }

        switch (type) {
            case "error":
                return ERROR_DURATION_MS;
            case "success":
                return SUCCESS_DURATION_MS;
            case "warning":
                return DEFAULT_DURATION_MS;
            case "info":
            default:
                return DEFAULT_DURATION_MS;
        }
    };

    /**
     * Schedule automatic dismissal for a notification
     *
     * @param id - Notification ID
     * @param duration - Dismiss delay in milliseconds
     */
    const scheduleAutoDismiss = (id: string, duration: number): void => {
        // Clear any existing timeout
        const existingTimeout = dismissTimeouts.get(id);
        if (existingTimeout) {
            clearTimeout(existingTimeout);
        }

        // Schedule new timeout
        const timeout = setTimeout(() => {
            dismissNotification(id);
            dismissTimeouts.delete(id);
        }, duration);

        dismissTimeouts.set(id, timeout);

        if (import.meta.env.DEV) {
            console.debug(
                `[useNotifications] Auto-dismiss scheduled for ${id} in ${duration}ms`,
            );
        }
    };

    /**
     * Enforce max notifications limit
     *
     * Removes oldest notifications if limit exceeded
     */
    const enforceMaxLimit = (): void => {
        if (notifications.value.length > MAX_NOTIFICATIONS) {
            const toRemove = notifications.value.length - MAX_NOTIFICATIONS;
            const removed = notifications.value.splice(0, toRemove);

            // Clear timeouts for removed notifications
            removed.forEach((notif) => {
                const timeout = dismissTimeouts.get(notif.id);
                if (timeout) {
                    clearTimeout(timeout);
                    dismissTimeouts.delete(notif.id);
                }
            });

            if (import.meta.env.DEV) {
                console.debug(
                    `[useNotifications] Removed ${toRemove} old notifications (limit: ${MAX_NOTIFICATIONS})`,
                );
            }
        }
    };

    // ==========================================
    // Core Functions
    // ==========================================

    /**
     * Show a notification
     *
     * Creates a new notification and schedules auto-dismissal.
     * Auto-dismisses after duration based on type.
     *
     * @param message - Notification message
     * @param type - Notification type (default: 'info')
     * @param duration - Custom duration in milliseconds (optional)
     * @returns Notification ID (can be used to dismiss manually)
     */
    const showNotification = (
        message: string,
        type: NotificationType = "info",
        duration?: number,
    ): string => {
        const id = generateId();
        const dismissDuration = getDuration(type, duration);

        const notification: Notification = {
            id,
            message,
            type,
            timestamp: Date.now(),
            duration: dismissDuration,
            dismissible: true,
        };

        // Add to notifications list
        notifications.value.push(notification);

        // Enforce max limit
        enforceMaxLimit();

        // Schedule auto-dismiss
        scheduleAutoDismiss(id, dismissDuration);

        if (import.meta.env.DEV) {
            console.debug("[useNotifications] Notification shown", {
                id,
                type,
                message: message.substring(0, 50),
                duration: dismissDuration,
            });
        }

        return id;
    };

    /**
     * Show success notification
     *
     * Convenience method for success messages
     *
     * @param message - Success message
     * @param duration - Custom duration (optional)
     * @returns Notification ID
     */
    const showSuccess = (message: string, duration?: number): string => {
        return showNotification(message, "success", duration);
    };

    /**
     * Show error notification
     *
     * Convenience method for error messages
     *
     * @param message - Error message
     * @param duration - Custom duration (optional)
     * @returns Notification ID
     */
    const showError = (message: string, duration?: number): string => {
        return showNotification(message, "error", duration);
    };

    /**
     * Show warning notification
     *
     * Convenience method for warning messages
     *
     * @param message - Warning message
     * @param duration - Custom duration (optional)
     * @returns Notification ID
     */
    const showWarning = (message: string, duration?: number): string => {
        return showNotification(message, "warning", duration);
    };

    /**
     * Show info notification
     *
     * Convenience method for info messages
     *
     * @param message - Info message
     * @param duration - Custom duration (optional)
     * @returns Notification ID
     */
    const showInfo = (message: string, duration?: number): string => {
        return showNotification(message, "info", duration);
    };

    /**
     * Dismiss a notification by ID
     *
     * Removes notification from queue and clears auto-dismiss timeout
     *
     * @param id - Notification ID to dismiss
     */
    const dismissNotification = (id: string): void => {
        // Clear auto-dismiss timeout if exists
        const timeout = dismissTimeouts.get(id);
        if (timeout) {
            clearTimeout(timeout);
            dismissTimeouts.delete(id);
        }

        // Remove from notifications list
        const index = notifications.value.findIndex((n) => n.id === id);
        if (index > -1) {
            notifications.value.splice(index, 1);

            if (import.meta.env.DEV) {
                console.debug("[useNotifications] Notification dismissed", { id });
            }
        }
    };

    /**
     * Dismiss all active notifications
     *
     * Clears entire notification queue
     */
    const dismissAll = (): void => {
        // Clear all timeouts
        dismissTimeouts.forEach((timeout) => {
            clearTimeout(timeout);
        });
        dismissTimeouts.clear();

        // Clear notifications
        const count = notifications.value.length;
        notifications.value = [];

        if (import.meta.env.DEV) {
            console.debug("[useNotifications] All notifications dismissed", { count });
        }
    };

    /**
     * Dismiss all notifications of a specific type
     *
     * @param type - Notification type to dismiss
     */
    const dismissByType = (type: NotificationType): void => {
        const toRemove = notifications.value.filter((n) => n.type === type);

        toRemove.forEach((notif) => {
            dismissNotification(notif.id);
        });

        if (import.meta.env.DEV) {
            console.debug("[useNotifications] Dismissed by type", {
                type,
                count: toRemove.length,
            });
        }
    };

    // ==========================================
    // Return Interface
    // ==========================================

    return {
        // State
        notifications,
        notificationCount,
        hasErrors,

        // Methods
        showNotification,
        showSuccess,
        showError,
        showWarning,
        showInfo,
        dismissNotification,
        dismissAll,
        dismissByType,
    };
}
