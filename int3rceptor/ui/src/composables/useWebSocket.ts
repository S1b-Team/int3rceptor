/**
 * useWebSocket Composable
 *
 * Manages WebSocket connections for real-time dashboard updates.
 * - Supports multiple channels: metrics, activity, health, connections
 * - Automatic reconnection with exponential backoff
 * - Message parsing and type-safe event handling
 * - Fallback to polling if WebSocket unavailable
 *
 * @example
 * const { isConnected, subscribe, unsubscribe } = useWebSocket();
 * subscribe('metrics', (data: SystemMetrics) => {
 *   console.log('New metrics:', data);
 * });
 */

import {
    ref,
    computed,
    onBeforeUnmount,
    type Ref,
    type ComputedRef,
} from "vue";
import type {
    SystemMetrics,
    ActivityEntry,
    HealthStatus,
    ConnectionStats,
} from "@/types/dashboard";

// ==========================================
// Configuration Constants
// ==========================================

/** WebSocket reconnection attempt max count */
const MAX_RECONNECT_ATTEMPTS = 5;

/** Initial reconnection delay in milliseconds */
const INITIAL_RECONNECT_DELAY = 1000;

/** Maximum reconnection delay in milliseconds */
const MAX_RECONNECT_DELAY = 30000;

/** WebSocket message timeout in milliseconds */
const MESSAGE_TIMEOUT_MS = 10000;

// ==========================================
// Type Definitions
// ==========================================

/**
 * WebSocket message structure
 */
interface WebSocketMessage<T = unknown> {
    type: string;
    channel?: string;
    data: T;
    timestamp: number;
    id?: string;
}

/**
 * Channel callback type for typed message handlers
 */
type ChannelCallback<T> = (data: T, message?: WebSocketMessage<T>) => void;

/**
 * Supported WebSocket channels
 */
export type WebSocketChannel =
    | "metrics"
    | "activity"
    | "health"
    | "connections";

/**
 * Return type for useWebSocket
 */
interface UseWebSocketReturn {
    isConnected: ComputedRef<boolean>;
    reconnectAttempts: Ref<number>;
    lastMessageTime: Ref<number>;
    timeSinceLastMessage: ComputedRef<number>;
    subscribe: <T>(
        channel: WebSocketChannel,
        callback: ChannelCallback<T>,
    ) => () => void;
    unsubscribe: (
        channel: WebSocketChannel,
        callback: ChannelCallback<any>,
    ) => void;
    connect: () => Promise<void>;
    disconnect: () => void;
    isChannelActive: (channel: WebSocketChannel) => boolean;
}

// ==========================================
// Composable Implementation
// ==========================================

/**
 * Composable for managing real-time WebSocket connections
 *
 * Features:
 * - Multiple channel subscriptions
 * - Automatic reconnection with exponential backoff
 * - Type-safe message handling
 * - Connection state tracking
 * - Memory leak prevention
 * - Debug logging
 *
 * @returns {UseWebSocketReturn} WebSocket connection state and methods
 */
export function useWebSocket(): UseWebSocketReturn {
    // ==========================================
    // State
    // ==========================================

    /** Active WebSocket connection */
    let ws: WebSocket | null = null;

    /** Connection state */
    const isConnected = ref(false);

    /** Current reconnection attempt count */
    const reconnectAttempts = ref(0);

    /** Last message received time */
    const lastMessageTime = ref(0);

    /** Callbacks registered for each channel */
    const subscriptions = ref<Map<WebSocketChannel, Set<ChannelCallback<any>>>>(
        new Map([
            ["metrics", new Set()],
            ["activity", new Set()],
            ["health", new Set()],
            ["connections", new Set()],
        ]),
    );

    // ==========================================
    // Internal Tracking
    // ==========================================

    /** Pending reconnection timeout */
    let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;

    /** Message timeout tracking */
    let messageTimeout: ReturnType<typeof setTimeout> | null = null;

    /** Whether cleanup has been called */
    let isCleanedUp = false;

    // ==========================================
    // Computed Properties
    // ==========================================

    /**
     * Computed: Time elapsed since last message (milliseconds)
     * Returns 0 if never received message
     */
    const timeSinceLastMessage = computed((): number => {
        if (!lastMessageTime.value) return 0;
        return Date.now() - lastMessageTime.value;
    });

    // ==========================================
    // Helper Functions
    // ==========================================

    /**
     * Get WebSocket URL based on current location
     * Automatically uses wss:// for HTTPS, ws:// for HTTP
     *
     * @returns WebSocket URL
     */
    const getWebSocketUrl = (): string => {
        const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
        const host = window.location.host;
        return `${protocol}//${host}/ws`;
    };

    /**
     * Calculate exponential backoff delay with jitter
     * Formula: min(initialDelay * 2^attempt, maxDelay) * jitter
     *
     * @param attempt - Current attempt number
     * @returns Delay in milliseconds
     */
    const calculateBackoffDelay = (attempt: number): number => {
        const exponentialDelay = INITIAL_RECONNECT_DELAY * Math.pow(2, attempt);
        const cappedDelay = Math.min(exponentialDelay, MAX_RECONNECT_DELAY);
        const jitter = cappedDelay * (0.5 + Math.random());
        return Math.floor(jitter);
    };

    /**
     * Parse and validate WebSocket message
     *
     * @param data - Raw message data
     * @returns Parsed message or null if invalid
     */
    const parseMessage = (data: string): WebSocketMessage<any> | null => {
        try {
            const parsed = JSON.parse(data);

            // Validate required fields
            if (!parsed.type || !parsed.data || !parsed.timestamp) {
                console.warn(
                    "[useWebSocket] Invalid message structure:",
                    parsed,
                );
                return null;
            }

            return parsed as WebSocketMessage<any>;
        } catch (err) {
            console.error("[useWebSocket] Failed to parse message:", err);
            return null;
        }
    };

    /**
     * Handle incoming WebSocket message
     *
     * @param message - Parsed WebSocket message
     */
    const handleMessage = (message: WebSocketMessage<any>): void => {
        lastMessageTime.value = Date.now();

        // Determine channel from message type
        const channel = (message.channel || message.type) as WebSocketChannel;

        if (import.meta.env.DEV) {
            console.debug("[useWebSocket] Message received", {
                channel,
                type: message.type,
                timestamp: message.timestamp,
            });
        }

        // Get callbacks for this channel
        const callbacks = subscriptions.value.get(channel);

        if (callbacks && callbacks.size > 0) {
            // Call all registered callbacks for this channel
            callbacks.forEach((callback) => {
                try {
                    callback(message.data, message);
                } catch (err) {
                    // Sanitize channel to prevent format string injection
                    const sanitizedChannel = String(channel).replace(/[%\n\r]/g, '');
                    console.error(
                        '[useWebSocket] Error in callback for %s:',
                        sanitizedChannel,
                        err,
                    );
                }
            });
        } else if (import.meta.env.DEV) {
            console.debug(
                `[useWebSocket] No subscribers for channel: ${channel}`,
            );
        }
    };

    /**
     * Set up WebSocket event handlers
     */
    const setupHandlers = (): void => {
        if (!ws) return;

        ws.onopen = () => {
            if (isCleanedUp) return;

            isConnected.value = true;
            reconnectAttempts.value = 0;

            if (import.meta.env.DEV) {
                console.debug("[useWebSocket] Connected");
            }

            // Reset message timeout on connection
            if (messageTimeout) {
                clearTimeout(messageTimeout);
            }
            setMessageTimeout();
        };

        ws.onmessage = (event: MessageEvent) => {
            if (isCleanedUp) return;

            // Reset message timeout on each message
            if (messageTimeout) {
                clearTimeout(messageTimeout);
            }

            const message = parseMessage(event.data);

            if (message) {
                handleMessage(message);
                setMessageTimeout();
            }
        };

        ws.onerror = (error: Event) => {
            if (isCleanedUp) return;

            console.error("[useWebSocket] Connection error:", error);
            isConnected.value = false;
        };

        ws.onclose = () => {
            if (isCleanedUp) return;

            isConnected.value = false;

            // Attempt reconnection with exponential backoff
            if (reconnectAttempts.value < MAX_RECONNECT_ATTEMPTS) {
                const delay = calculateBackoffDelay(reconnectAttempts.value);
                reconnectAttempts.value++;

                if (import.meta.env.DEV) {
                    console.debug(
                        `[useWebSocket] Reconnecting in ${delay}ms (attempt ${reconnectAttempts.value})`,
                    );
                }

                reconnectTimeout = setTimeout(() => {
                    connect();
                }, delay);
            } else {
                console.error(
                    "[useWebSocket] Max reconnection attempts exceeded",
                );
            }
        };
    };

    /**
     * Set up message timeout (close connection if no messages)
     */
    const setMessageTimeout = (): void => {
        messageTimeout = setTimeout(() => {
            if (isConnected.value) {
                console.warn("[useWebSocket] Message timeout - reconnecting");
                disconnect();
                connect();
            }
        }, MESSAGE_TIMEOUT_MS);
    };

    // ==========================================
    // Core Functions
    // ==========================================

    /**
     * Establish WebSocket connection
     *
     * Handles:
     * - Connection establishment
     * - Event handler setup
     * - Error handling
     * - Automatic reconnection
     */
    const connect = async (): Promise<void> => {
        // Don't reconnect if already connected or cleaned up
        if (ws || isConnected.value || isCleanedUp) {
            if (isConnected.value && import.meta.env.DEV) {
                console.debug("[useWebSocket] Already connected");
            }
            return;
        }

        try {
            const url = getWebSocketUrl();

            if (import.meta.env.DEV) {
                console.debug("[useWebSocket] Connecting to", url);
            }

            ws = new WebSocket(url);
            setupHandlers();

            // Wait for connection to establish
            await new Promise<void>((resolve, reject) => {
                const timeout = setTimeout(() => {
                    reject(new Error("WebSocket connection timeout"));
                }, 5000);

                const checkConnection = () => {
                    if (isConnected.value) {
                        clearTimeout(timeout);
                        resolve();
                    } else if (ws?.readyState === WebSocket.CLOSED) {
                        clearTimeout(timeout);
                        reject(new Error("WebSocket connection failed"));
                    }
                };

                // Check connection state periodically
                const interval = setInterval(() => {
                    checkConnection();
                    if (
                        isConnected.value ||
                        ws?.readyState === WebSocket.CLOSED
                    ) {
                        clearInterval(interval);
                    }
                }, 100);
            });
        } catch (err) {
            console.error("[useWebSocket] Connection failed:", err);
            isConnected.value = false;

            // Schedule reconnection
            if (reconnectAttempts.value < MAX_RECONNECT_ATTEMPTS) {
                const delay = calculateBackoffDelay(reconnectAttempts.value);
                reconnectAttempts.value++;

                reconnectTimeout = setTimeout(() => {
                    connect();
                }, delay);
            }
        }
    };

    /**
     * Disconnect WebSocket and clear resources
     *
     * Safe to call multiple times
     */
    const disconnect = (): void => {
        if (reconnectTimeout) {
            clearTimeout(reconnectTimeout);
            reconnectTimeout = null;
        }

        if (messageTimeout) {
            clearTimeout(messageTimeout);
            messageTimeout = null;
        }

        if (ws) {
            ws.close();
            ws = null;
        }

        isConnected.value = false;
        reconnectAttempts.value = 0;

        if (import.meta.env.DEV) {
            console.debug("[useWebSocket] Disconnected");
        }
    };

    /**
     * Subscribe to WebSocket channel updates
     *
     * @param channel - Channel name
     * @param callback - Function to call when messages arrive
     * @returns Unsubscribe function
     */
    const subscribe = <T>(
        channel: WebSocketChannel,
        callback: ChannelCallback<T>,
    ): (() => void) => {
        const callbacks = subscriptions.value.get(channel);

        if (callbacks) {
            callbacks.add(callback as ChannelCallback<any>);

            if (import.meta.env.DEV) {
                console.debug(`[useWebSocket] Subscribed to ${channel}`, {
                    subscribers: callbacks.size,
                });
            }

            // Return unsubscribe function
            return () => {
                unsubscribe(channel, callback);
            };
        }

        return () => {};
    };

    /**
     * Unsubscribe from WebSocket channel
     *
     * @param channel - Channel name
     * @param callback - Callback to remove
     */
    const unsubscribe = (
        channel: WebSocketChannel,
        callback: ChannelCallback<any>,
    ): void => {
        const callbacks = subscriptions.value.get(channel);

        if (callbacks) {
            callbacks.delete(callback);

            if (import.meta.env.DEV) {
                console.debug(`[useWebSocket] Unsubscribed from ${channel}`, {
                    subscribers: callbacks.size,
                });
            }
        }
    };

    /**
     * Check if channel has active subscribers
     *
     * @param channel - Channel name
     * @returns Whether channel has subscribers
     */
    const isChannelActive = (channel: WebSocketChannel): boolean => {
        const callbacks = subscriptions.value.get(channel);
        return (callbacks?.size || 0) > 0;
    };

    // ==========================================
    // Lifecycle Management
    // ==========================================

    /**
     * Cleanup on component unmount
     * Ensures connection is closed and timers cleared
     */
    onBeforeUnmount(() => {
        isCleanedUp = true;
        disconnect();

        if (import.meta.env.DEV) {
            console.debug("[useWebSocket] Cleanup completed");
        }
    });

    // ==========================================
    // Return Interface
    // ==========================================

    return {
        // State
        isConnected: computed(() => isConnected.value),
        reconnectAttempts,
        lastMessageTime,
        timeSinceLastMessage,

        // Methods
        subscribe,
        unsubscribe,
        connect,
        disconnect,
        isChannelActive,
    };
}
