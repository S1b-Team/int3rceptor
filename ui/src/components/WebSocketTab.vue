<script setup lang="ts">
/**
 * WebSocketTab.vue - VOIDWALKER WebSocket Analysis Component
 *
 * Displays WebSocket connections and frames with real-time updates.
 */

import { ref, computed, onMounted, onUnmounted } from "vue";

// Types for VOIDWALKER WebSocket data
interface ConnectionSummary {
    id: string;
    http_request_id: number;
    url: string;
    host: string;
    protocol?: string;
    state: string;
    frames_sent: number;
    frames_received: number;
    bytes_sent: number;
    bytes_received: number;
    started_at: string;
    ended_at?: string;
    secure: boolean;
}

interface WebSocketFrame {
    id: number;
    connection_id: string;
    direction: "sent" | "received";
    frame_type: string;
    payload: number[];
    text?: string;
    length: number;
    masked: boolean;
    fin: boolean;
    compressed: boolean;
    timestamp: string;
    notes?: string;
    highlight?: string;
}

interface FrameDiff {
    frame_a_id: number;
    frame_b_id: number;
    changes: DiffChange[];
    insertions: number;
    deletions: number;
    similarity: number;
}

interface DiffChange {
    change_type: "equal" | "insert" | "delete";
    content: string;
    old_line?: number;
    new_line?: number;
}

// Props
const props = defineProps<{
    apiBase?: string;
}>();

const apiBase = props.apiBase || "http://localhost:7071";

// State
const connections = ref<ConnectionSummary[]>([]);
const selectedConnection = ref<ConnectionSummary | null>(null);
const frames = ref<WebSocketFrame[]>([]);
const selectedFrame = ref<WebSocketFrame | null>(null);
const compareFrame = ref<WebSocketFrame | null>(null);
const diffResult = ref<FrameDiff | null>(null);
const loading = ref(false);
const viewMode = ref<"list" | "compare">("list");

// Direction colors
const directionColors = {
    sent: "#00d4ff",
    received: "#ff006e",
};

// Frame type colors
const frameTypeColors: Record<string, string> = {
    text: "#ffffff",
    binary: "#8b5cf6",
    close: "#ff0000",
    ping: "#ffb800",
    pong: "#ffb800",
    continuation: "#606060",
};

// Connection state colors
const stateColors: Record<string, string> = {
    connecting: "#ffb800",
    open: "#00ff00",
    closing: "#ffb800",
    closed: "#606060",
    failed: "#ff0000",
};

// Fetch connections
async function fetchConnections() {
    loading.value = true;
    try {
        const url = `${apiBase}/api/websocket/connections`;
        const res = await fetch(url);
        if (res.ok) {
            connections.value = await res.json();
        }
    } catch (e) {
        console.error("Failed to fetch connections:", e);
    } finally {
        loading.value = false;
    }
}

// Fetch frames for a connection
async function fetchFrames(connectionId: string) {
    try {
        const url = `${apiBase}/api/websocket/connections/${connectionId}/frames`;
        const res = await fetch(url);
        if (res.ok) {
            frames.value = await res.json();
        }
    } catch (e) {
        console.error("Failed to fetch frames:", e);
    }
}

// Select connection
function selectConnection(conn: ConnectionSummary) {
    selectedConnection.value = conn;
    selectedFrame.value = null;
    compareFrame.value = null;
    diffResult.value = null;
    viewMode.value = "list";
    fetchFrames(conn.id);
}

// Select frame
function selectFrame(frame: WebSocketFrame) {
    if (
        viewMode.value === "compare" &&
        selectedFrame.value &&
        selectedFrame.value.id !== frame.id
    ) {
        compareFrame.value = frame;
        compareTwoFrames();
    } else {
        selectedFrame.value = frame;
        compareFrame.value = null;
        diffResult.value = null;
    }
}

// Compare two frames
async function compareTwoFrames() {
    if (!selectedFrame.value || !compareFrame.value) return;

    try {
        const url = `${apiBase}/api/websocket/frames/compare?a=${selectedFrame.value.id}&b=${compareFrame.value.id}`;
        const res = await fetch(url);
        if (res.ok) {
            diffResult.value = await res.json();
        }
    } catch (e) {
        console.error("Failed to compare frames:", e);
    }
}

// Enable compare mode
function enableCompareMode() {
    viewMode.value = "compare";
    compareFrame.value = null;
    diffResult.value = null;
}

// Exit compare mode
function exitCompareMode() {
    viewMode.value = "list";
    compareFrame.value = null;
    diffResult.value = null;
}

// Format timestamp
function formatTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString("en-US", {
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
        fractionalSecondDigits: 3,
    });
}

// Format bytes
function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// Get direction icon
function getDirectionIcon(direction: string): string {
    return direction === "sent" ? "→" : "←";
}

// Get frame preview
function getFramePreview(frame: WebSocketFrame, maxLen: number = 60): string {
    if (frame.text) {
        return frame.text.length <= maxLen
            ? frame.text
            : frame.text.substring(0, maxLen) + "...";
    }
    return `<binary ${frame.length} bytes>`;
}

// Try to pretty-print JSON
function prettyPrintJSON(text: string | undefined): string {
    if (!text) return "";
    try {
        const parsed = JSON.parse(text);
        return JSON.stringify(parsed, null, 2);
    } catch {
        return text;
    }
}

// Render diff with colors
function renderDiff(diff: FrameDiff): string {
    return diff.changes
        .map((change) => {
            let color: string;
            let prefix: string;

            switch (change.change_type) {
                case "insert":
                    color = "#00ff00";
                    prefix = "+";
                    break;
                case "delete":
                    color = "#ff0000";
                    prefix = "-";
                    break;
                default:
                    color = "#888";
                    prefix = " ";
            }

            return `<span style="color: ${color}">${prefix}${escapeHtml(
                change.content
            )}</span>`;
        })
        .join("");
}

// Escape HTML
function escapeHtml(text: string): string {
    return text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;");
}

// WebSocket for real-time updates
let ws: WebSocket | null = null;

function connectWebSocket() {
    const wsUrl = apiBase.replace("http", "ws") + "/ws/events";
    ws = new WebSocket(wsUrl);

    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        if (
            data.type === "connection_opened" ||
            data.type === "connection_closed"
        ) {
            fetchConnections();
        } else if (
            data.type === "frame_captured" &&
            selectedConnection.value?.id === data.connection_id
        ) {
            fetchFrames(selectedConnection.value.id);
        }
    };

    ws.onclose = () => {
        setTimeout(connectWebSocket, 5000);
    };
}

// Lifecycle
onMounted(() => {
    fetchConnections();
    connectWebSocket();
});

onUnmounted(() => {
    ws?.close();
});
</script>

<template>
    <div class="websocket-tab">
        <!-- Header -->
        <div class="ws-header">
            <h2 class="ws-title">
                <span class="icon">🌐</span> WebSocket Analysis
            </h2>

            <div class="header-actions">
                <button
                    class="refresh-btn"
                    @click="fetchConnections"
                    :disabled="loading"
                >
                    {{ loading ? "Loading..." : "Refresh" }}
                </button>
            </div>
        </div>

        <!-- Main content -->
        <div class="ws-content">
            <!-- Connections panel -->
            <div class="connections-panel">
                <h3 class="panel-title">Connections</h3>

                <div class="connections-list">
                    <div
                        v-for="conn in connections"
                        :key="conn.id"
                        :class="[
                            'connection-item',
                            { selected: selectedConnection?.id === conn.id },
                        ]"
                        @click="selectConnection(conn)"
                    >
                        <div class="conn-header">
                            <span class="conn-id">{{ conn.id }}</span>
                            <span
                                class="conn-state"
                                :style="{
                                    color: stateColors[conn.state] || '#888',
                                }"
                            >
                                {{ conn.state.toUpperCase() }}
                            </span>
                        </div>

                        <div class="conn-url">{{ conn.url }}</div>

                        <div class="conn-stats">
                            <span class="sent">↑ {{ conn.frames_sent }}</span>
                            <span class="received"
                                >↓ {{ conn.frames_received }}</span
                            >
                            <span class="bytes">{{
                                formatBytes(
                                    conn.bytes_sent + conn.bytes_received
                                )
                            }}</span>
                        </div>
                    </div>

                    <div v-if="connections.length === 0" class="empty-state">
                        No WebSocket connections
                    </div>
                </div>
            </div>

            <!-- Frames panel -->
            <div class="frames-panel" v-if="selectedConnection">
                <div class="frames-header">
                    <h3 class="panel-title">Frames</h3>

                    <div class="frames-actions">
                        <button
                            v-if="viewMode === 'list'"
                            class="action-btn"
                            @click="enableCompareMode"
                            :disabled="frames.length < 2"
                        >
                            Compare
                        </button>
                        <button
                            v-else
                            class="action-btn active"
                            @click="exitCompareMode"
                        >
                            Exit Compare
                        </button>
                    </div>
                </div>

                <div v-if="viewMode === 'compare'" class="compare-hint">
                    Select two frames to compare
                </div>

                <div class="frames-list">
                    <div
                        v-for="frame in frames"
                        :key="frame.id"
                        :class="[
                            'frame-item',
                            {
                                selected: selectedFrame?.id === frame.id,
                                'compare-selected':
                                    compareFrame?.id === frame.id,
                            },
                        ]"
                        @click="selectFrame(frame)"
                    >
                        <span
                            class="frame-direction"
                            :style="{ color: directionColors[frame.direction] }"
                        >
                            {{ getDirectionIcon(frame.direction) }}
                        </span>

                        <span
                            class="frame-type"
                            :style="{
                                color:
                                    frameTypeColors[frame.frame_type] || '#fff',
                            }"
                        >
                            {{ frame.frame_type }}
                        </span>

                        <span class="frame-preview">{{
                            getFramePreview(frame)
                        }}</span>

                        <span class="frame-length">{{ frame.length }}B</span>

                        <span class="frame-time">{{
                            formatTime(frame.timestamp)
                        }}</span>
                    </div>

                    <div v-if="frames.length === 0" class="empty-state">
                        No frames captured
                    </div>
                </div>
            </div>

            <!-- Detail panel -->
            <div class="detail-panel" v-if="selectedFrame">
                <div class="detail-header">
                    <span
                        class="direction-badge"
                        :style="{
                            backgroundColor:
                                directionColors[selectedFrame.direction],
                        }"
                    >
                        {{ selectedFrame.direction.toUpperCase() }}
                    </span>

                    <span class="frame-info">
                        {{ selectedFrame.frame_type.toUpperCase() }} •
                        {{ selectedFrame.length }} bytes •
                        {{ formatTime(selectedFrame.timestamp) }}
                    </span>
                </div>

                <!-- Compare view -->
                <div v-if="diffResult" class="diff-view">
                    <div class="diff-header">
                        <span
                            >Comparing Frame #{{ selectedFrame.id }} vs #{{
                                compareFrame?.id
                            }}</span
                        >
                        <span class="diff-stats">
                            +{{ diffResult.insertions }} -{{
                                diffResult.deletions
                            }}
                            ({{ (diffResult.similarity * 100).toFixed(1) }}%
                            similar)
                        </span>
                    </div>
                    <pre
                        class="diff-content"
                        v-html="renderDiff(diffResult)"
                    ></pre>
                </div>

                <!-- Single frame view -->
                <div v-else class="frame-content">
                    <pre
                        v-if="selectedFrame.frame_type === 'text'"
                        class="text-content"
                        >{{ prettyPrintJSON(selectedFrame.text) }}</pre
                    >

                    <div v-else class="hex-view">
                        <span class="hex-label"
                            >Binary Data ({{
                                selectedFrame.length
                            }}
                            bytes)</span
                        >
                        <!-- Hex dump would go here -->
                    </div>
                </div>
            </div>

            <!-- Empty detail panel -->
            <div class="detail-panel empty" v-else-if="selectedConnection">
                <span class="empty-hint">Select a frame to view details</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.websocket-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #0a0a0a;
    color: #ffffff;
    font-family: "JetBrains Mono", "Fira Code", monospace;
}

.ws-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #1a1a1a;
    background: linear-gradient(180deg, #0f0f0f 0%, #0a0a0a 100%);
}

.ws-title {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #00d4ff;
    display: flex;
    align-items: center;
    gap: 8px;
}

.icon {
    font-size: 20px;
}

.refresh-btn {
    background: linear-gradient(90deg, #00d4ff, #00b4d8);
    color: #000;
    border: none;
    padding: 8px 16px;
    font-size: 13px;
    font-weight: 600;
    border-radius: 4px;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
}

.refresh-btn:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 212, 255, 0.3);
}

.refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.ws-content {
    display: flex;
    flex: 1;
    overflow: hidden;
}

.connections-panel {
    width: 280px;
    border-right: 1px solid #1a1a1a;
    display: flex;
    flex-direction: column;
}

.panel-title {
    margin: 0;
    padding: 12px 16px;
    font-size: 12px;
    color: #888;
    text-transform: uppercase;
    border-bottom: 1px solid #1a1a1a;
}

.connections-list {
    flex: 1;
    overflow: auto;
}

.connection-item {
    padding: 12px 16px;
    border-bottom: 1px solid #1a1a1a;
    cursor: pointer;
    transition: background 0.2s;
}

.connection-item:hover {
    background: #1a1a1a;
}

.connection-item.selected {
    background: rgba(0, 212, 255, 0.1);
}

.conn-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 4px;
}

.conn-id {
    color: #00d4ff;
    font-size: 12px;
}

.conn-state {
    font-size: 10px;
    font-weight: 600;
}

.conn-url {
    font-size: 11px;
    color: #888;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 8px;
}

.conn-stats {
    display: flex;
    gap: 12px;
    font-size: 11px;
}

.conn-stats .sent {
    color: #00d4ff;
}
.conn-stats .received {
    color: #ff006e;
}
.conn-stats .bytes {
    color: #888;
}

.frames-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    border-right: 1px solid #1a1a1a;
}

.frames-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-right: 16px;
}

.action-btn {
    background: transparent;
    border: 1px solid #2a2a2a;
    color: #888;
    padding: 4px 12px;
    font-size: 11px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
}

.action-btn:hover:not(:disabled) {
    border-color: #00d4ff;
    color: #00d4ff;
}

.action-btn.active {
    background: rgba(139, 92, 246, 0.2);
    border-color: #8b5cf6;
    color: #8b5cf6;
}

.compare-hint {
    padding: 8px 16px;
    background: rgba(139, 92, 246, 0.1);
    color: #8b5cf6;
    font-size: 12px;
}

.frames-list {
    flex: 1;
    overflow: auto;
}

.frame-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    border-bottom: 1px solid #1a1a1a;
    cursor: pointer;
    font-size: 12px;
    transition: background 0.2s;
}

.frame-item:hover {
    background: #1a1a1a;
}

.frame-item.selected {
    background: rgba(0, 212, 255, 0.1);
}

.frame-item.compare-selected {
    background: rgba(139, 92, 246, 0.1);
}

.frame-direction {
    font-weight: 600;
}

.frame-type {
    min-width: 60px;
}

.frame-preview {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: #888;
}

.frame-length {
    color: #606060;
    min-width: 50px;
    text-align: right;
}

.frame-time {
    color: #606060;
    min-width: 100px;
    text-align: right;
}

.detail-panel {
    width: 40%;
    min-width: 350px;
    display: flex;
    flex-direction: column;
    background: #0f0f0f;
}

.detail-panel.empty {
    align-items: center;
    justify-content: center;
}

.empty-hint {
    color: #606060;
    font-size: 14px;
}

.detail-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid #2a2a2a;
}

.direction-badge {
    color: #000;
    padding: 2px 8px;
    font-size: 10px;
    font-weight: 700;
    border-radius: 2px;
}

.frame-info {
    color: #888;
    font-size: 12px;
}

.diff-view {
    flex: 1;
    overflow: auto;
    padding: 16px;
}

.diff-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 12px;
    font-size: 12px;
    color: #888;
}

.diff-stats {
    color: #8b5cf6;
}

.diff-content {
    background: #0a0a0a;
    border: 1px solid #2a2a2a;
    border-radius: 4px;
    padding: 12px;
    margin: 0;
    font-size: 11px;
    line-height: 1.6;
    overflow: auto;
}

.frame-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
}

.text-content {
    background: #0a0a0a;
    border: 1px solid #2a2a2a;
    border-radius: 4px;
    padding: 12px;
    margin: 0;
    font-size: 12px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-all;
}

.hex-view {
    background: #0a0a0a;
    border: 1px solid #2a2a2a;
    border-radius: 4px;
    padding: 12px;
}

.hex-label {
    color: #8b5cf6;
    font-size: 12px;
}

.empty-state {
    padding: 24px;
    text-align: center;
    color: #606060;
    font-size: 13px;
}
</style>
