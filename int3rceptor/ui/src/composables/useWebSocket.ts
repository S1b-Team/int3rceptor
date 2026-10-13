/**
 * useWebSocket Composable
 *
 * Manages the WebSocket connection to the backend `/ws` stream.
 * - Exposes `connected` and the latest captured request (`latest`)
 * - Supports channels: request, hello, metrics, activity, health, connections
 * - Automatic reconnection with exponential backoff
 * - Message parsing and type-safe event handling
 * - Fallback to polling if WebSocket unavailable
 *
 * @example
 * const { connected, latest, subscribe } = useWebSocket("ws://127.0.0.1:3000/ws");
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
import type { CaptureEntry } from "@/types";

// ==========================================
// Configuration Constants
// ==========================================


/** Initial reconnection delay in milliseconds */
const INITIAL_RECONNECT_DELAY = 1000;

/** Maximum reconnection delay in milliseconds */
const MAX_RECONNECT_DELAY = 30000;


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
    timestamp?: number;
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
    | "request"
    | "hello"
    | "metrics"
    | "activity"
    | "health"
    | "connections";

/**
 * Return type for useWebSocket
 */
export interface UseWebSocketReturn {
    /** True while the socket is open */
    connected: ComputedRef<boolean>;
    /** Alias of `connected`, kept for existing callers */
    isConnected: ComputedRef<boolean>;
    /** Most recent captured request pushed by the backend */
    latest: Ref<CaptureEntry | null>;
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
export function useWebSocket(url?: string): UseWebSocketReturn {
    // ==========================================
    // State
    // ==========================================

    /** Active WebSocket connection */
    let ws: WebSocket | null = null;

    /** Connection state */
    const isConnected = ref(false);

    /** Current reconnection attempt count */
    const reconnectAttempts = ref(0);

    /** Latest captured request received on the "request" channel */
    const latest = ref<CaptureEntry | null>(null) as Ref<CaptureEntry | null>;

    /** Last message received time */
    const lastMessageTime = ref(0);

    /** Callbacks registered for each channel */
    const subscriptions = ref<Map<WebSocketChannel, Set<ChannelCallback<any>>>>(
        new Map([
            ["request", new Set()],
            ["hello", new Set()],
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

    /** Set when disconnect() is called on purpose, to skip auto-reconnect */
    let manualClose = false;

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
        if (url) return url;
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
            if (!parsed || typeof parsed.type !== "string" || parsed.data === undefined) {
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

        if (channel === "request") {
            latest.value = message.data as CaptureEntry;
        }

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
        };

        ws.onmessage = (event: MessageEvent) => {
            if (isCleanedUp) return;

            const message = parseMessage(event.data);

            if (message) {
                handleMessage(message);
            }
        };

        ws.onerror = (error: Event) => {
            if (isCleanedUp) return;

            console.error("[useWebSocket] Connection error:", error);
            isConnected.value = false;
        };

        ws.onclose = () => {
            ws = null;
            isConnected.value = false;
            if (isCleanedUp || manualClose) return;

            // Keep retrying with capped exponential backoff; the proxy may
            // simply not be running yet.
            {
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
            }
        };
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
        manualClose = false;
        if (reconnectTimeout) {
            clearTimeout(reconnectTimeout);
            reconnectTimeout = null;
        }
        if (ws || isConnected.value || isCleanedUp) {
            if (isConnected.value && import.meta.env.DEV) {
                console.debug("[useWebSocket] Already connected");
            }
            return;
        }

        try {
            const wsUrl = getWebSocketUrl();

            if (import.meta.env.DEV) {
                console.debug("[useWebSocket] Connecting to", wsUrl);
            }

            ws = new WebSocket(wsUrl);
            setupHandlers();
        } catch (err) {
            // Invalid URL etc. onclose won't fire, so schedule a retry here.
            console.error("[useWebSocket] Connection failed:", err);
            ws = null;
            isConnected.value = false;
            const delay = calculateBackoffDelay(reconnectAttempts.value);
            reconnectAttempts.value++;
            reconnectTimeout = setTimeout(() => {
                connect();
            }, delay);
        }
    };

    /**
     * Disconnect WebSocket and clear resources
     *
     * Safe to call multiple times
     */
    const disconnect = (): void => {
        manualClose = true;
        if (reconnectTimeout) {
            clearTimeout(reconnectTimeout);
            reconnectTimeout = null;
        }

        if (ws) {
            const socket = ws;
            ws = null;
            socket.close();
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

    // Start connecting right away; callers just read `connected` / `latest`.
    void connect();

    const connectedRef = computed(() => isConnected.value);

    return {
        // State
        connected: connectedRef,
        isConnected: connectedRef,
        latest,
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
