<script setup lang="ts">
/**
 * TrafficTab.vue - NOWARU Traffic Intelligence Component
 *
 * Displays HTTP traffic with syntax highlighting, filtering, and analysis.
 */

import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import CodeViewer from "./CodeViewer.vue";

// Types for NOWARU traffic data
interface ParsedHeader {
    name: string;
    value: string;
}

interface ParsedRequest {
    id: number;
    timestamp: string;
    method: string;
    url: string;
    path: string;
    query?: string;
    host: string;
    http_version: string;
    headers: ParsedHeader[];
    content_type?: string;
    content_category: string;
    body_text?: string;
    body_size: number;
    tls: boolean;
    cookies: Record<string, string>;
}

interface ParsedResponse {
    request_id: number;
    status_code: number;
    status_text: string;
    status_category: string;
    http_version: string;
    headers: ParsedHeader[];
    content_type?: string;
    content_category: string;
    body_text?: string;
    body_size: number;
    duration_ms: number;
    set_cookies: string[];
}

interface HttpTransaction {
    id: number;
    request: ParsedRequest;
    response?: ParsedResponse;
    tags: string[];
    notes?: string;
    highlight?: string;
}

interface SyntaxToken {
    type: string;
    content: string;
    line: number;
    column: number;
}

interface HighlightedContent {
    raw: string;
    tokens: SyntaxToken[];
    category: string;
    line_count: number;
}

// Props
const props = defineProps<{
    apiBase?: string;
}>();

const apiBase = props.apiBase || "http://localhost:7070";

// State
const transactions = ref<HttpTransaction[]>([]);
const selectedTransaction = ref<HttpTransaction | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const activeView = ref<"request" | "response">("request");
const highlightedContent = ref<HighlightedContent | null>(null);

// Filters
const filters = ref({
    method: "",
    host: "",
    status: "",
    search: "",
    contentType: "",
});

// Method colors matching UI spec
const methodColors: Record<string, string> = {
    GET: "#00ff00",
    POST: "#00d4ff",
    PUT: "#ffb800",
    PATCH: "#ffb800",
    DELETE: "#ff0000",
    HEAD: "#8b5cf6",
    OPTIONS: "#8b5cf6",
};

// Status category colors
const statusColors: Record<string, string> = {
    informational: "#8b5cf6",
    success: "#00ff00",
    redirect: "#00d4ff",
    client_error: "#ffb800",
    server_error: "#ff0000",
};

// Fetch transactions
async function fetchTransactions() {
    loading.value = true;
    error.value = null;

    try {
        const params = new URLSearchParams();
        if (filters.value.method) params.set("method", filters.value.method);
        if (filters.value.host) params.set("host", filters.value.host);
        if (filters.value.status) params.set("status", filters.value.status);
        if (filters.value.search) params.set("search", filters.value.search);
        params.set("limit", "500");

        const url = `${apiBase}/api/traffic?${params}`;
        const res = await fetch(url);

        if (!res.ok) {
            throw new Error(`HTTP ${res.status}: ${res.statusText}`);
        }

        transactions.value = await res.json();
    } catch (e: any) {
        error.value = e.message;
        console.error("Failed to fetch transactions:", e);
    } finally {
        loading.value = false;
    }
}

// Fetch highlighted content
async function fetchHighlighted(id: number, type: "request" | "response") {
    try {
        const url = `${apiBase}/api/traffic/${id}/${type}/highlighted`;
        const res = await fetch(url);

        if (res.ok) {
            highlightedContent.value = await res.json();
        }
    } catch (e) {
        console.error("Failed to fetch highlighted content:", e);
    }
}

// Select a transaction
function selectTransaction(tx: HttpTransaction) {
    selectedTransaction.value = tx;
    activeView.value = "request";
    fetchHighlighted(tx.id, "request");
}

// Switch between request/response view
function switchView(view: "request" | "response") {
    if (!selectedTransaction.value) return;

    activeView.value = view;
    fetchHighlighted(selectedTransaction.value.id, view);
}

// Format timestamp
function formatTime(timestamp: string): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString("en-US", {
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
    });
}

// Format size
function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// Get method color
function getMethodColor(method: string): string {
    return methodColors[method.toUpperCase()] || "#ffffff";
}

// Get status color
function getStatusColor(category?: string): string {
    return statusColors[category || "success"] || "#ffffff";
}

// Render syntax-highlighted content
function renderHighlighted(content: HighlightedContent): string {
    if (!content.tokens.length) {
        return escapeHtml(content.raw);
    }

    let result = "";
    let lastEnd = 0;
    const raw = content.raw;

    // Sort tokens by position
    const sorted = [...content.tokens].sort((a, b) => {
        if (a.line !== b.line) return a.line - b.line;
        return a.column - b.column;
    });

    // Find token positions and wrap with color spans
    for (const token of sorted) {
        const color = getTokenColor(token.type);
        result += `<span style="color: ${color}">${escapeHtml(
            token.content
        )}</span>`;
    }

    return result || escapeHtml(raw);
}

// Get color for token type
function getTokenColor(type: string): string {
    const colors: Record<string, string> = {
        key: "#00d4ff",
        string: "#ff006e",
        number: "#ffb800",
        boolean: "#8b5cf6",
        null: "#606060",
        punctuation: "#a0a0a0",
        tag: "#00d4ff",
        attribute: "#8b5cf6",
        attribute_value: "#ff006e",
        comment: "#606060",
        keyword: "#00d4ff",
        operator: "#a0a0a0",
        function: "#ffb800",
        selector: "#00d4ff",
        property: "#8b5cf6",
        text: "#ffffff",
        error: "#ff0000",
    };
    return colors[type] || "#ffffff";
}

// Escape HTML
function escapeHtml(text: string): string {
    return text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
}

// WebSocket for real-time updates
let ws: WebSocket | null = null;

function connectWebSocket() {
    const wsUrl = apiBase.replace("http", "ws") + "/ws/events";
    ws = new WebSocket(wsUrl);

    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        if (data.type === "new_capture" || data.type === "capture_updated") {
            fetchTransactions();
        }
    };

    ws.onclose = () => {
        setTimeout(connectWebSocket, 5000);
    };

    ws.onerror = (e) => {
        console.error("WebSocket error:", e);
    };
}

// Lifecycle
onMounted(() => {
    fetchTransactions();
    connectWebSocket();
});

onUnmounted(() => {
    ws?.close();
});

// Watch filters
watch(
    filters,
    () => {
        fetchTransactions();
    },
    { deep: true }
);
</script>

<template>
    <div class="traffic-tab">
        <!-- Header with filters -->
        <div class="traffic-header">
            <h2 class="traffic-title">
                <span class="icon">📡</span> Traffic Intelligence
            </h2>

            <div class="filters">
                <select v-model="filters.method" class="filter-select">
                    <option value="">All Methods</option>
                    <option value="GET">GET</option>
                    <option value="POST">POST</option>
                    <option value="PUT">PUT</option>
                    <option value="PATCH">PATCH</option>
                    <option value="DELETE">DELETE</option>
                </select>

                <input
                    v-model="filters.host"
                    type="text"
                    placeholder="Host..."
                    class="filter-input"
                />

                <input
                    v-model="filters.search"
                    type="text"
                    placeholder="Search..."
                    class="filter-input"
                />

                <button
                    class="refresh-btn"
                    @click="fetchTransactions"
                    :disabled="loading"
                >
                    {{ loading ? "Loading..." : "Refresh" }}
                </button>
            </div>
        </div>

        <!-- Error message -->
        <div v-if="error" class="error-message">
            {{ error }}
        </div>

        <!-- Main content area -->
        <div class="traffic-content">
            <!-- Transaction list -->
            <div class="transaction-list">
                <table class="traffic-table">
                    <thead>
                        <tr>
                            <th class="col-id">#</th>
                            <th class="col-time">Time</th>
                            <th class="col-method">Method</th>
                            <th class="col-host">Host</th>
                            <th class="col-path">Path</th>
                            <th class="col-status">Status</th>
                            <th class="col-size">Size</th>
                            <th class="col-duration">Time</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr
                            v-for="tx in transactions"
                            :key="tx.id"
                            :class="{
                                selected: selectedTransaction?.id === tx.id,
                            }"
                            @click="selectTransaction(tx)"
                        >
                            <td class="col-id">{{ tx.id }}</td>
                            <td class="col-time">
                                {{ formatTime(tx.request.timestamp) }}
                            </td>
                            <td class="col-method">
                                <span
                                    :style="{
                                        color: getMethodColor(
                                            tx.request.method
                                        ),
                                    }"
                                >
                                    {{ tx.request.method }}
                                </span>
                            </td>
                            <td class="col-host">{{ tx.request.host }}</td>
                            <td class="col-path" :title="tx.request.url">
                                {{ tx.request.path }}
                            </td>
                            <td class="col-status">
                                <span
                                    v-if="tx.response"
                                    :style="{
                                        color: getStatusColor(
                                            tx.response.status_category
                                        ),
                                    }"
                                >
                                    {{ tx.response.status_code }}
                                </span>
                                <span v-else class="pending">...</span>
                            </td>
                            <td class="col-size">
                                {{
                                    tx.response
                                        ? formatSize(tx.response.body_size)
                                        : "-"
                                }}
                            </td>
                            <td class="col-duration">
                                {{
                                    tx.response
                                        ? `${tx.response.duration_ms}ms`
                                        : "-"
                                }}
                            </td>
                        </tr>
                    </tbody>
                </table>

                <div
                    v-if="!loading && transactions.length === 0"
                    class="empty-state"
                >
                    No traffic captured yet
                </div>
            </div>

            <!-- Detail panel -->
            <div class="detail-panel" v-if="selectedTransaction">
                <div class="detail-header">
                    <div class="view-tabs">
                        <button
                            :class="{ active: activeView === 'request' }"
                            @click="switchView('request')"
                        >
                            Request
                        </button>
                        <button
                            :class="{ active: activeView === 'response' }"
                            @click="switchView('response')"
                            :disabled="!selectedTransaction.response"
                        >
                            Response
                        </button>
                    </div>

                    <div class="detail-info">
                        <span
                            :style="{
                                color: getMethodColor(
                                    selectedTransaction.request.method
                                ),
                            }"
                        >
                            {{ selectedTransaction.request.method }}
                        </span>
                        <span class="url">{{
                            selectedTransaction.request.url
                        }}</span>
                    </div>
                </div>

                <!-- Request view -->
                <div v-if="activeView === 'request'" class="detail-content">
                    <div class="section">
                        <h3>Headers</h3>
                        <div class="headers-list">
                            <div
                                v-for="header in selectedTransaction.request
                                    .headers"
                                :key="header.name"
                                class="header-row"
                            >
                                <span class="header-name"
                                    >{{ header.name }}:</span
                                >
                                <span class="header-value">{{
                                    header.value
                                }}</span>
                            </div>
                        </div>
                    </div>

                    <div
                        class="section"
                        v-if="selectedTransaction.request.body_text"
                    >
                        <h3>
                            Body
                            <span class="content-type">{{
                                selectedTransaction.request.content_category
                            }}</span>
                        </h3>
                        <pre
                            class="body-content"
                            v-html="
                                highlightedContent
                                    ? renderHighlighted(highlightedContent)
                                    : selectedTransaction.request.body_text
                            "
                        ></pre>
                    </div>
                </div>

                <!-- Response view -->
                <div
                    v-else-if="
                        activeView === 'response' &&
                        selectedTransaction.response
                    "
                    class="detail-content"
                >
                    <div class="section">
                        <h3>
                            Status:
                            <span
                                :style="{
                                    color: getStatusColor(
                                        selectedTransaction.response
                                            .status_category
                                    ),
                                }"
                            >
                                {{ selectedTransaction.response.status_code }}
                                {{ selectedTransaction.response.status_text }}
                            </span>
                        </h3>
                    </div>

                    <div class="section">
                        <h3>Headers</h3>
                        <div class="headers-list">
                            <div
                                v-for="header in selectedTransaction.response
                                    .headers"
                                :key="header.name"
                                class="header-row"
                            >
                                <span class="header-name"
                                    >{{ header.name }}:</span
                                >
                                <span class="header-value">{{
                                    header.value
                                }}</span>
                            </div>
                        </div>
                    </div>

                    <div
                        class="section"
                        v-if="selectedTransaction.response.body_text"
                    >
                        <h3>
                            Body
                            <span class="content-type">{{
                                selectedTransaction.response.content_category
                            }}</span>
                        </h3>
                        <pre
                            class="body-content"
                            v-html="
                                highlightedContent
                                    ? renderHighlighted(highlightedContent)
                                    : selectedTransaction.response.body_text
                            "
                        ></pre>
                    </div>
                </div>
            </div>

            <!-- Empty state for detail panel -->
            <div class="detail-panel empty" v-else>
                <div class="empty-detail">Select a request to view details</div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.traffic-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #0a0a0a;
    color: #ffffff;
    font-family: "JetBrains Mono", "Fira Code", monospace;
}

.traffic-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid #1a1a1a;
    background: linear-gradient(180deg, #0f0f0f 0%, #0a0a0a 100%);
}

.traffic-title {
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

.filters {
    display: flex;
    gap: 12px;
}

.filter-select,
.filter-input {
    background: #1a1a1a;
    border: 1px solid #2a2a2a;
    color: #ffffff;
    padding: 8px 12px;
    font-size: 13px;
    border-radius: 4px;
    transition: border-color 0.2s;
}

.filter-select:focus,
.filter-input:focus {
    outline: none;
    border-color: #00d4ff;
}

.filter-input {
    width: 150px;
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

.error-message {
    background: rgba(255, 0, 0, 0.1);
    border: 1px solid #ff0000;
    color: #ff6666;
    padding: 12px;
    margin: 16px;
    border-radius: 4px;
}

.traffic-content {
    display: flex;
    flex: 1;
    overflow: hidden;
}

.transaction-list {
    flex: 1;
    overflow: auto;
    border-right: 1px solid #1a1a1a;
}

.traffic-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
}

.traffic-table th {
    position: sticky;
    top: 0;
    background: #0f0f0f;
    text-align: left;
    padding: 10px 12px;
    color: #888;
    font-weight: 500;
    border-bottom: 1px solid #2a2a2a;
    z-index: 1;
}

.traffic-table td {
    padding: 8px 12px;
    border-bottom: 1px solid #1a1a1a;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.traffic-table tr:hover {
    background: #1a1a1a;
}

.traffic-table tr.selected {
    background: rgba(0, 212, 255, 0.1);
}

.traffic-table tr.selected:hover {
    background: rgba(0, 212, 255, 0.15);
}

.col-id {
    width: 50px;
    color: #606060;
}
.col-time {
    width: 80px;
    color: #888;
}
.col-method {
    width: 70px;
    font-weight: 600;
}
.col-host {
    width: 150px;
}
.col-path {
    max-width: 300px;
}
.col-status {
    width: 60px;
    font-weight: 600;
    text-align: center;
}
.col-size {
    width: 70px;
    text-align: right;
    color: #888;
}
.col-duration {
    width: 70px;
    text-align: right;
    color: #888;
}

.pending {
    color: #606060;
}

.empty-state {
    padding: 40px;
    text-align: center;
    color: #606060;
}

.detail-panel {
    width: 45%;
    min-width: 400px;
    display: flex;
    flex-direction: column;
    background: #0f0f0f;
}

.detail-panel.empty {
    align-items: center;
    justify-content: center;
}

.empty-detail {
    color: #606060;
    font-size: 14px;
}

.detail-header {
    padding: 12px 16px;
    border-bottom: 1px solid #2a2a2a;
}

.view-tabs {
    display: flex;
    gap: 8px;
    margin-bottom: 12px;
}

.view-tabs button {
    background: transparent;
    border: 1px solid #2a2a2a;
    color: #888;
    padding: 6px 16px;
    font-size: 12px;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
}

.view-tabs button:hover:not(:disabled) {
    border-color: #00d4ff;
    color: #00d4ff;
}

.view-tabs button.active {
    background: rgba(0, 212, 255, 0.1);
    border-color: #00d4ff;
    color: #00d4ff;
}

.view-tabs button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
}

.detail-info {
    font-size: 13px;
    display: flex;
    gap: 12px;
    align-items: center;
}

.detail-info .url {
    color: #888;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.detail-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
}

.section {
    margin-bottom: 20px;
}

.section h3 {
    font-size: 12px;
    color: #888;
    text-transform: uppercase;
    margin: 0 0 12px 0;
    padding-bottom: 8px;
    border-bottom: 1px solid #2a2a2a;
}

.content-type {
    color: #8b5cf6;
    font-weight: normal;
    margin-left: 8px;
}

.headers-list {
    font-size: 12px;
}

.header-row {
    display: flex;
    padding: 4px 0;
    gap: 8px;
}

.header-name {
    color: #00d4ff;
    min-width: 150px;
}

.header-value {
    color: #ffffff;
    word-break: break-all;
}

.body-content {
    background: #0a0a0a;
    border: 1px solid #2a2a2a;
    border-radius: 4px;
    padding: 12px;
    margin: 0;
    font-size: 12px;
    line-height: 1.5;
    overflow: auto;
    max-height: 400px;
    white-space: pre-wrap;
    word-break: break-all;
}
</style>
