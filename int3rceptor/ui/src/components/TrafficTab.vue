<script setup lang="ts">
/**
 * TrafficTab.vue - captured HTTP traffic view.
 *
 * Presentational: the parent owns data loading, filters and the live
 * websocket feed. This component renders the filter bar, the request
 * table, the details panel and the repeater.
 */
import RequestDetail from "./RequestDetail.vue";
import RepeaterTab from "./RepeaterTab.vue";
import type { CaptureEntry } from "@/types";

export interface TrafficFilters {
    method: string;
    host: string;
    status: string;
    tls: "all" | "https" | "http";
    search: string;
}

export type ExportFormat = "json" | "csv" | "har";

const props = defineProps<{
    items: CaptureEntry[];
    selectedId: number | null;
    selectedEntry: CaptureEntry | null;
    filters: TrafficFilters;
    exporting?: boolean;
}>();

const emit = defineEmits<{
    select: [number];
    "update:filters": [TrafficFilters];
    export: [ExportFormat];
}>();

function setFilter<K extends keyof TrafficFilters>(
    key: K,
    value: TrafficFilters[K]
) {
    emit("update:filters", { ...props.filters, [key]: value });
}

function parseUrl(raw: string): { host: string; path: string } {
    try {
        const u = new URL(raw);
        return { host: u.host, path: `${u.pathname}${u.search}` || "/" };
    } catch {
        return { host: "", path: raw };
    }
}

function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString("en-US", {
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
    });
}

function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function methodClass(method: string): string {
    const m = method.toUpperCase();
    if (m === "GET") return "method-get";
    if (m === "POST") return "method-post";
    if (m === "PUT" || m === "PATCH") return "method-put";
    if (m === "DELETE") return "method-delete";
    return "method-other";
}

function statusClass(code: number): string {
    if (code >= 500) return "http-5xx";
    if (code >= 400) return "http-4xx";
    if (code >= 300) return "http-3xx";
    return "http-2xx";
}
</script>

<template>
    <div class="traffic-tab">
        <section class="traffic-filters">
            <div class="field">
                <label for="tf-method">Method</label>
                <select
                    id="tf-method"
                    :value="filters.method"
                    @change="setFilter('method', ($event.target as HTMLSelectElement).value)"
                >
                    <option value="">All</option>
                    <option>GET</option>
                    <option>POST</option>
                    <option>PUT</option>
                    <option>DELETE</option>
                    <option>PATCH</option>
                </select>
            </div>
            <div class="field">
                <label for="tf-host">Host</label>
                <input
                    id="tf-host"
                    :value="filters.host"
                    placeholder="example.com"
                    @input="setFilter('host', ($event.target as HTMLInputElement).value)"
                />
            </div>
            <div class="field">
                <label for="tf-status">Status</label>
                <input
                    id="tf-status"
                    :value="filters.status"
                    placeholder="200"
                    @input="setFilter('status', ($event.target as HTMLInputElement).value)"
                />
            </div>
            <div class="field">
                <label for="tf-tls">TLS</label>
                <select
                    id="tf-tls"
                    :value="filters.tls"
                    @change="setFilter('tls', ($event.target as HTMLSelectElement).value as TrafficFilters['tls'])"
                >
                    <option value="all">All</option>
                    <option value="https">HTTPS</option>
                    <option value="http">HTTP</option>
                </select>
            </div>
            <div class="field">
                <label for="tf-search">Search</label>
                <input
                    id="tf-search"
                    :value="filters.search"
                    placeholder="/api"
                    @input="setFilter('search', ($event.target as HTMLInputElement).value)"
                />
            </div>
            <div class="field">
                <label>Export</label>
                <div class="export-buttons">
                    <button
                        v-for="fmt in (['json', 'csv', 'har'] as ExportFormat[])"
                        :key="fmt"
                        class="ghost"
                        :disabled="exporting"
                        @click="emit('export', fmt)"
                    >
                        {{ fmt.toUpperCase() }}
                    </button>
                </div>
            </div>
        </section>

        <div class="traffic-content">
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
                            <th class="col-duration">Duration</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr
                            v-for="entry in items"
                            :key="entry.request.id"
                            :class="{ selected: entry.request.id === selectedId }"
                            tabindex="0"
                            @click="emit('select', entry.request.id)"
                            @keydown.enter="emit('select', entry.request.id)"
                        >
                            <td class="col-id">{{ entry.request.id }}</td>
                            <td class="col-time">
                                {{ formatTime(entry.request.timestamp_ms) }}
                            </td>
                            <td class="col-method">
                                <span :class="['pill', methodClass(entry.request.method)]">
                                    {{ entry.request.method }}
                                </span>
                            </td>
                            <td class="col-host">
                                <span v-if="entry.request.tls" class="tls" title="HTTPS">🔒</span>
                                {{ parseUrl(entry.request.url).host }}
                            </td>
                            <td class="col-path" :title="entry.request.url">
                                {{ parseUrl(entry.request.url).path }}
                            </td>
                            <td class="col-status">
                                <span
                                    v-if="entry.response"
                                    :class="['pill', statusClass(entry.response.status_code)]"
                                >
                                    {{ entry.response.status_code }}
                                </span>
                                <span v-else class="pending">…</span>
                            </td>
                            <td class="col-size">
                                {{ entry.response ? formatSize(entry.response.body.length) : "-" }}
                            </td>
                            <td class="col-duration">
                                {{ entry.response ? `${entry.response.duration_ms} ms` : "-" }}
                            </td>
                        </tr>
                        <tr v-if="!items.length" class="empty-row">
                            <td colspan="8" class="empty-state">
                                No requests captured yet.
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <aside class="detail-panel">
                <RequestDetail :entry="selectedEntry" />
            </aside>
        </div>

        <section class="repeater-panel">
            <RepeaterTab :entry="selectedEntry" />
        </section>
    </div>
</template>

<style scoped>
.traffic-tab {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
}

/* ---------- Filters ---------- */
.traffic-filters {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: var(--spacing-md);
    padding: var(--spacing-md) var(--spacing-xl);
    border-bottom: var(--border-width-thin) solid var(--color-border-default);
}

.field label {
    display: block;
    font-size: var(--font-size-xs);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-secondary);
    margin-bottom: var(--spacing-xs);
}

.field input,
.field select {
    width: 100%;
    box-sizing: border-box;
    background: var(--color-bg-secondary);
    border: var(--border-width-thin) solid var(--color-border-default);
    border-radius: var(--border-radius-sm);
    padding: 6px var(--spacing-sm);
    color: var(--color-text-primary);
    font-family: var(--font-sans);
    transition: border-color var(--transition-fast);
}

.field input::placeholder {
    color: var(--color-text-muted);
}

.field input:focus,
.field select:focus {
    outline: 2px solid var(--color-border-active);
    outline-offset: 1px;
    border-color: var(--color-border-active);
}

.export-buttons {
    display: flex;
    gap: 6px;
}

.ghost {
    background: transparent;
    border: var(--border-width-thin) solid var(--color-accent-cyan);
    color: var(--color-accent-cyan);
    padding: 5px var(--spacing-sm);
    border-radius: var(--border-radius-sm);
    font-family: var(--font-mono);
    font-size: var(--font-size-xs);
    cursor: pointer;
    transition: background var(--transition-fast),
        box-shadow var(--transition-fast);
}

.ghost:hover:not(:disabled) {
    background: rgba(0, 212, 255, 0.1);
    box-shadow: var(--shadow-cyan);
}

.ghost:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

/* ---------- Layout ---------- */
.traffic-content {
    display: flex;
    flex: 1;
    min-height: 320px;
    overflow: hidden;
}

.transaction-list {
    flex: 1;
    min-width: 0;
    overflow: auto;
    border-right: var(--border-width-thin) solid var(--color-border-default);
}

.detail-panel {
    width: 42%;
    min-width: 360px;
    overflow: auto;
    background: var(--color-bg-secondary);
}

.repeater-panel {
    padding: var(--spacing-md);
    border-top: var(--border-width-thin) solid var(--color-border-default);
}

/* ---------- Table ---------- */
.traffic-table {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--font-mono);
    font-size: var(--font-size-sm);
    table-layout: fixed;
}

.traffic-table th {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--color-bg-primary);
    text-align: left;
    padding: 10px 12px;
    color: var(--color-accent-cyan);
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    border-bottom: var(--border-width-thin) solid var(--color-border-default);
}

.traffic-table td {
    padding: var(--spacing-sm) 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.traffic-table tbody tr {
    cursor: pointer;
    background: var(--color-bg-primary);
    box-shadow: inset 0 0 0 transparent;
    transition: background var(--transition-fast),
        box-shadow var(--transition-fast);
}

.traffic-table tbody tr:nth-child(even) {
    background: var(--color-row-alt);
}

.traffic-table tbody tr:hover,
.traffic-table tbody tr:focus-visible {
    outline: none;
    box-shadow: inset 3px 0 0 var(--color-accent-cyan);
}

.traffic-table tbody tr.selected {
    background: var(--color-row-selected);
    box-shadow: inset 5px 0 0 var(--color-accent-cyan);
}

.traffic-table tbody tr.empty-row {
    cursor: default;
    box-shadow: none;
}

.col-id {
    width: 56px;
    color: var(--color-text-muted);
}
.col-time {
    width: 84px;
    color: var(--color-text-secondary);
}
.col-method {
    width: 84px;
}
.col-host {
    width: 180px;
}
.col-status {
    width: 72px;
}
.col-size,
.col-duration {
    width: 84px;
    text-align: right;
    color: var(--color-text-secondary);
}

.tls {
    font-size: var(--font-size-xs);
    margin-right: 2px;
}

.pending,
.empty-state {
    color: var(--color-text-muted);
}

.empty-state {
    padding: var(--spacing-3xl) 0;
    text-align: center;
}

/* ---------- Badges ---------- */
.pill {
    display: inline-block;
    min-width: 44px;
    text-align: center;
    padding: 2px var(--spacing-sm);
    border-radius: var(--border-radius-full);
    font-size: var(--font-size-xs);
    font-weight: var(--font-weight-semibold);
    border: var(--border-width-thin) solid currentColor;
    background: color-mix(in srgb, currentColor 12%, transparent);
}

.method-get { color: var(--color-method-get); }
.method-post { color: var(--color-method-post); }
.method-put { color: var(--color-method-put); }
.method-delete { color: var(--color-method-delete); }
.method-other { color: var(--color-method-other); }

.http-2xx { color: var(--color-http-2xx); }
.http-3xx { color: var(--color-http-3xx); }
.http-4xx { color: var(--color-http-4xx); }
.http-5xx { color: var(--color-http-5xx); }
</style>
