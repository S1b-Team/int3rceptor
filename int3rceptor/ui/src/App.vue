<template>
    <div class="app-shell">
        <header>
            <h1 class="brand">Int3rceptor</h1>
            <div class="status-bar">
                <span :class="['pill', ws.connected ? 'ok' : 'warn']">
                    WS {{ ws.connected ? "Connected" : "Disconnected" }}
                </span>
                <button @click="refresh" :disabled="loading">Refresh</button>
                <button @click="clear" :disabled="loading || !items.length">
                    Clear
                </button>
            </div>
        </header>

        <div class="ca-tip">
            <div>
                <strong>Install the Int3rceptor CA</strong>
                <p>
                    Run
                    <code>interceptor --export-ca ./interceptor-ca.pem</code>
                    and trust it in your OS/browser before intercepting HTTPS
                    traffic.
                </p>
            </div>
            <button class="secondary" @click="downloadCa">Download CA</button>
        </div>

        <nav class="main-nav">
            <button
                :class="{ active: currentTab === 'dashboard' }"
                @click="currentTab = 'dashboard'"
            >
                Dashboard
            </button>
            <button
                :class="{ active: currentTab === 'traffic' }"
                @click="currentTab = 'traffic'"
            >
                Traffic
            </button>
            <button
                :class="{ active: currentTab === 'rules' }"
                @click="currentTab = 'rules'"
            >
                Rules
            </button>
            <button
                :class="{ active: currentTab === 'scope' }"
                @click="currentTab = 'scope'"
            >
                Scope
            </button>
            <button
                :class="{ active: currentTab === 'intruder' }"
                @click="currentTab = 'intruder'"
            >
                Intruder
            </button>
            <button
                :class="{ active: currentTab === 'intercept' }"
                @click="currentTab = 'intercept'"
            >
                Intercept
            </button>
            <button
                :class="{ active: currentTab === 'websocket' }"
                @click="currentTab = 'websocket'"
            >
                WebSocket
            </button>
        </nav>

        <template v-if="currentTab === 'traffic'">
            <TrafficTab
                v-model:filters="filters"
                :items="items"
                :selected-id="selectedId"
                :selected-entry="selectedEntry"
                :exporting="exporting"
                @select="handleSelect"
                @export="exportData"
            />
        </template>

        <template v-else-if="currentTab === 'rules'">
            <RulesTab />
        </template>

        <template v-else-if="currentTab === 'scope'">
            <ScopeTab />
        </template>

        <template v-else-if="currentTab === 'intruder'">
            <IntruderTab :entry="selectedEntry" />
        </template>

        <template v-else-if="currentTab === 'intercept'">
            <InterceptTab />
        </template>

        <template v-else-if="currentTab === 'websocket'">
            <WebSocketTab :api-base="api.defaults.baseURL ?? 'http://127.0.0.1:3000'" />
        </template>

        <template v-else>
            <DashboardTab />
        </template>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import TrafficTab from "./components/TrafficTab.vue";
import type { TrafficFilters, ExportFormat } from "./components/TrafficTab.vue";
import RulesTab from "./components/RulesTab.vue";
import ScopeTab from "./components/ScopeTab.vue";
import IntruderTab from "./components/IntruderTab.vue";
import DashboardTab from "./components/DashboardTab.vue";
import InterceptTab from "./components/InterceptTab.vue";
import WebSocketTab from "./components/WebSocketTab.vue";
import { useApi, api } from "./composables/useApi";
import { useWebSocket } from "./composables/useWebSocket";
import type { CaptureEntry } from "./types";

const { listRequests, clearRequests, exportRequests } = useApi();
const wsUrl = import.meta.env.VITE_WS_URL ?? "ws://127.0.0.1:3000/ws";
const ws = useWebSocket(wsUrl);
const items = ref<CaptureEntry[]>([]);
const selectedId = ref<number | null>(null);
const loading = ref(false);
const exporting = ref(false);
const currentTab = ref<
    | "traffic"
    | "rules"
    | "scope"
    | "intruder"
    | "intercept"
    | "websocket"
    | "dashboard"
>("traffic");

const filters = ref<TrafficFilters>({
    method: "",
    host: "",
    status: "",
    tls: "all",
    search: "",
});

const filterParams = computed(() => {
    const f = filters.value;
    const params: Record<string, string> = {};
    if (f.method) params.method = f.method;
    if (f.host) params.host = f.host;
    if (f.status) params.status = f.status;
    if (f.tls === "https") params.tls = "true";
    if (f.tls === "http") params.tls = "false";
    if (f.search) params.search = f.search;
    return params;
});

const selectedEntry = computed(
    () =>
        items.value.find((entry) => entry.request.id === selectedId.value) ??
        null
);

const refresh = async () => {
    loading.value = true;
    try {
        items.value = await listRequests(filterParams.value);
    } finally {
        loading.value = false;
    }
};

const clear = async () => {
    await clearRequests();
    items.value = [];
    selectedId.value = null;
    refresh();
};

const handleSelect = (id: number) => {
    selectedId.value = id;
};

const downloadCa = () => {
    const base = api.defaults.baseURL ?? "http://127.0.0.1:3000";
    window.open(`${base}/api/ca-cert`, "_blank");
};

watch(
    () => ws.latest.value,
    (incoming) => {
        if (!incoming) {
            return;
        }
        items.value = [incoming, ...items.value].slice(0, 500);
    }
);

onMounted(() => {
    refresh();
});

watch(filterParams, () => {
    refresh();
});

const exportData = async (format: ExportFormat) => {
    exporting.value = true;
    try {
        const blob = await exportRequests(format, filterParams.value);
        const urlObject = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = urlObject;
        link.download = `interceptor.${format === "har" ? "har" : format}`;
        document.body.appendChild(link);
        link.click();
        link.remove();
        URL.revokeObjectURL(urlObject);
    } finally {
        exporting.value = false;
    }
};
</script>

<style scoped>
.app-shell {
    font-family: var(--font-sans);
    min-height: 100vh;
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    display: flex;
    flex-direction: column;
}

/* ---------- Header ---------- */
header {
    height: 60px;
    box-sizing: border-box;
    padding: 0 var(--spacing-xl);
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: var(--color-bg-secondary);
    border-bottom: var(--border-width-thin) solid var(--color-border-default);
}

.brand {
    margin: 0;
    font-family: var(--font-heading);
    font-size: var(--font-size-xl);
    font-weight: var(--font-weight-bold);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--color-accent-cyan);
    text-shadow: var(--shadow-cyan);
}

.status-bar {
    display: flex;
    gap: var(--spacing-sm);
    align-items: center;
}

/* ---------- Navigation ---------- */
.main-nav {
    display: flex;
    gap: var(--spacing-xs);
    padding: 0 var(--spacing-xl);
    background: var(--color-bg-primary);
    border-bottom: var(--border-width-thin) solid var(--color-border-default);
}

.main-nav button {
    background: transparent;
    color: var(--color-text-secondary);
    border: none;
    border-bottom: var(--border-width-medium) solid transparent;
    border-radius: 0;
    padding: var(--spacing-md) var(--spacing-md) calc(var(--spacing-md) - 2px);
    font-family: var(--font-sans);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
    letter-spacing: 0.02em;
    box-shadow: none;
    transition: color var(--transition-fast),
        border-color var(--transition-fast);
}

.main-nav button:hover:not(.active) {
    color: var(--color-text-primary);
    box-shadow: none;
}

.main-nav button.active {
    color: var(--color-accent-cyan);
    border-bottom-color: var(--color-accent-cyan);
}

/* ---------- CA tip ---------- */
.ca-tip {
    margin: var(--spacing-md) var(--spacing-xl);
    padding: var(--spacing-md);
    background: var(--color-bg-soft);
    border: var(--border-width-thin) solid var(--color-border-default);
    border-left: var(--border-width-thick) solid var(--color-accent-orange);
    border-radius: var(--border-radius-lg);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--spacing-md);
}

.ca-tip strong {
    color: var(--color-accent-orange);
}

.ca-tip p {
    margin: var(--spacing-xs) 0 0;
    color: var(--color-text-secondary);
    font-size: var(--font-size-sm);
}

.ca-tip code {
    color: var(--color-accent-cyan);
    background: var(--color-bg-primary);
    padding: 1px 6px;
    border-radius: var(--border-radius-sm);
}

/* ---------- Buttons ---------- */
button {
    background: var(--color-accent-cyan);
    color: var(--color-bg-primary);
    border: none;
    padding: var(--spacing-sm) var(--spacing-md);
    border-radius: var(--border-radius-md);
    cursor: pointer;
    font-family: var(--font-sans);
    font-weight: var(--font-weight-semibold);
    transition: box-shadow var(--transition-fast),
        background var(--transition-fast),
        color var(--transition-fast);
}

button:hover:not(:disabled) {
    box-shadow: var(--shadow-cyan-lg);
}

button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.secondary {
    background: transparent;
    border: var(--border-width-thin) solid var(--color-accent-cyan);
    color: var(--color-accent-cyan);
}

.secondary:hover:not(:disabled) {
    background: rgba(0, 212, 255, 0.1);
}

/* ---------- Status pill ---------- */
.pill {
    padding: var(--spacing-xs) 10px;
    border-radius: var(--border-radius-full);
    font-size: var(--font-size-xs);
    font-family: var(--font-mono);
}

.pill.ok {
    background: rgba(34, 197, 94, 0.15);
    color: var(--color-status-success);
}

.pill.warn {
    background: rgba(255, 0, 110, 0.15);
    color: var(--color-accent-magenta);
}
</style>
