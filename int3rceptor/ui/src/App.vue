<template>
    <div class="app-shell">
        <header>
            <h1>Interceptor</h1>
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
                <strong>Install the Interceptor CA</strong>
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
        </nav>

        <template v-if="currentTab === 'traffic'">
            <section class="filters">
                <div>
                    <label>Method</label>
                    <select v-model="filterMethod">
                        <option value="">All</option>
                        <option>GET</option>
                        <option>POST</option>
                        <option>PUT</option>
                        <option>DELETE</option>
                        <option>PATCH</option>
                    </select>
                </div>
                <div>
                    <label>Host</label>
                    <input v-model="filterHost" placeholder="example.com" />
                </div>
                <div>
                    <label>Status</label>
                    <input v-model="filterStatus" placeholder="200" />
                </div>
                <div>
                    <label>TLS</label>
                    <select v-model="filterTls">
                        <option value="all">All</option>
                        <option value="https">HTTPS</option>
                        <option value="http">HTTP</option>
                    </select>
                </div>
                <div>
                    <label>Search</label>
                    <input v-model="filterSearch" placeholder="/api" />
                </div>
                <div class="export">
                    <label>Export</label>
                    <div class="export-buttons">
                        <button
                            :disabled="exporting"
                            @click="exportData('json')"
                        >
                            JSON
                        </button>
                        <button
                            :disabled="exporting"
                            @click="exportData('csv')"
                        >
                            CSV
                        </button>
                        <button
                            :disabled="exporting"
                            @click="exportData('har')"
                        >
                            HAR
                        </button>
                    </div>
                </div>
            </section>

            <main>
                <section class="list-panel">
                    <RequestList
                        :items="items"
                        :selected-id="selectedId"
                        @select="handleSelect"
                    />
                </section>
                <section class="detail-panel">
                    <RequestDetail :entry="selectedEntry" />
                </section>
            </main>

            <section class="repeater-panel">
                <RepeaterTab :entry="selectedEntry" />
            </section>
        </template>

        <template v-else-if="currentTab === 'rules'">
            <RulesTab />
        </template>

        <template v-else-if="currentTab === 'scope'">
            <ScopeTab />
        </template>

        <template v-else-if="currentTab === 'intruder'">
            <IntruderTab />
        </template>

        <template v-else>
            <DashboardTab />
        </template>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import RequestList from "./components/RequestList.vue";
import RequestDetail from "./components/RequestDetail.vue";
import RepeaterTab from "./components/RepeaterTab.vue";
import RulesTab from "./components/RulesTab.vue";
import ScopeTab from "./components/ScopeTab.vue";
import IntruderTab from "./components/IntruderTab.vue";
import DashboardTab from "./components/DashboardTab.vue";
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
    "traffic" | "rules" | "scope" | "intruder" | "dashboard"
>("traffic");

const filterMethod = ref("");
const filterHost = ref("");
const filterStatus = ref("");
const filterTls = ref("all");
const filterSearch = ref("");

const filterParams = computed(() => {
    const params: Record<string, string> = {};
    if (filterMethod.value) params.method = filterMethod.value;
    if (filterHost.value) params.host = filterHost.value;
    if (filterStatus.value) params.status = filterStatus.value;
    if (filterTls.value === "https") params.tls = "true";
    if (filterTls.value === "http") params.tls = "false";
    if (filterSearch.value) params.search = filterSearch.value;
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

const exportData = async (format: "json" | "csv" | "har") => {
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
    font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI",
        sans-serif;
    min-height: 100vh;
    background: #0f172a;
    color: #e2e8f0;
    display: flex;
    flex-direction: column;
}

.main-nav {
    display: flex;
    gap: 1rem;
    padding: 0.5rem 2rem;
    background: rgba(15, 23, 42, 0.95);
    border-bottom: 1px solid rgba(148, 163, 184, 0.2);
}

.main-nav button {
    background: transparent;
    color: #94a3b8;
    border: 1px solid transparent;
    padding: 0.4rem 1rem;
    cursor: pointer;
    font-weight: 600;
    border-radius: 4px;
}

.main-nav button.active {
    background: rgba(56, 189, 248, 0.1);
    color: #38bdf8;
    border-color: rgba(56, 189, 248, 0.3);
}

header {
    padding: 1rem 2rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(148, 163, 184, 0.3);
}

.status-bar {
    display: flex;
    gap: 0.5rem;
    align-items: center;
}

.filters {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
    gap: 0.75rem;
    padding: 1rem 2rem;
}

.filters label {
    display: block;
    font-size: 0.8rem;
    text-transform: uppercase;
    color: #94a3b8;
    margin-bottom: 0.2rem;
}

.filters input,
.filters select {
    width: 100%;
    background: rgba(15, 23, 42, 0.8);
    border: 1px solid rgba(148, 163, 184, 0.2);
    border-radius: 4px;
    padding: 0.4rem;
    color: #e2e8f0;
}

.export {
    display: flex;
    flex-direction: column;
}

.export-buttons {
    display: flex;
    gap: 0.4rem;
}

.ca-tip {
    margin: 0 2rem;
    padding: 0.75rem 1rem;
    border: 1px solid rgba(56, 189, 248, 0.2);
    border-radius: 8px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
}

.ca-tip p {
    margin: 0.25rem 0 0;
    color: #94a3b8;
}

.secondary {
    background: transparent;
    border: 1px solid rgba(56, 189, 248, 0.6);
    color: #bae6fd;
}

button {
    background: #38bdf8;
    border: none;
    padding: 0.4rem 0.8rem;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 600;
}

button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

main {
    display: grid;
    grid-template-columns: 1fr 1fr;
    flex: 1;
    min-height: 0;
}

.list-panel,
.detail-panel,
.repeater-panel {
    border-top: 1px solid rgba(148, 163, 184, 0.2);
}

.list-panel,
.detail-panel {
    overflow: hidden;
}

.repeater-panel {
    padding: 1rem;
}

.pill {
    padding: 0.25rem 0.6rem;
    border-radius: 999px;
    font-size: 0.85rem;
}

.pill.ok {
    background: rgba(34, 197, 94, 0.2);
    color: #4ade80;
}

.pill.warn {
    background: rgba(248, 113, 113, 0.2);
    color: #fca5a5;
}
</style>
