<template>
    <div class="recent-activity-panel">
        <!-- Panel Header -->
        <div class="panel-header">
            <h2 class="panel-title">📋 Recent Activity</h2>
            <div class="header-controls">
                <button class="btn-icon" @click="refreshActivity" title="Refresh">🔄</button>
                <button class="btn-icon" @click="clearActivity" title="Clear">🗑️</button>
            </div>
        </div>

        <!-- Activity List -->
        <div class="activity-list">
            <div v-if="activities.length === 0" class="empty-state">
                <span class="empty-icon">📭</span>
                <p>No recent activity</p>
            </div>

            <div v-else class="activity-timeline">
                <div
                    v-for="(activity, index) in activities"
                    :key="activity.id"
                    class="activity-entry"
                    :class="activity.type"
                >
                    <!-- Timeline Dot -->
                    <div class="timeline-dot">
                        <span class="dot-icon">{{ getActivityIcon(activity.type) }}</span>
                    </div>

                    <!-- Activity Content -->
                    <div class="activity-content">
                        <div class="activity-header">
                            <span class="activity-type-badge" :class="activity.type">
                                {{ activity.type.toUpperCase() }}
                            </span>
                            <span class="activity-timestamp">{{
                                formatActivityTime(activity.timestamp)
                            }}</span>
                        </div>
                        <div class="activity-message">{{ activity.message }}</div>
                        <div v-if="activity.details" class="activity-details">
                            {{ activity.details }}
                        </div>
                    </div>

                    <!-- Activity Action -->
                    <div class="activity-action">
                        <button
                            v-if="activity.actionLabel"
                            class="btn-action"
                            @click="handleAction(activity)"
                        >
                            {{ activity.actionLabel }}
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Load More -->
        <div v-if="hasMore" class="load-more">
            <button class="btn-secondary" @click="loadMore">Load More Activity</button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import type { ActivityEntry } from "@/types/dashboard";

interface ExtendedActivityEntry {
    id: string;
    timestamp: number;
    type: "request" | "error" | "warning" | "info" | "response" | "success";
    message: string;
    actionLabel?: string;
    details?: string;
}

interface Props {
    activities?: ExtendedActivityEntry[];
    maxDisplay?: number;
}

const props = withDefaults(defineProps<Props>(), {
    maxDisplay: 10,
});

const emit = defineEmits<{
    refresh: [];
    clear: [];
    action: [activity: ExtendedActivityEntry];
}>();

// State
const displayedCount = ref(props.maxDisplay);

// Computed
const displayedActivities = computed(() => {
    return (props.activities ?? []).slice(0, displayedCount.value);
});

const hasMore = computed(() => {
    return (props.activities?.length ?? 0) > displayedCount.value;
});

const activities = computed(() => {
    return displayedActivities.value;
});

// Methods
const getActivityIcon = (type: string): string => {
    const icons: Record<string, string> = {
        request: "📨",
        response: "📤",
        error: "❌",
        warning: "⚠️",
        info: "ⓘ",
        success: "✓",
        proxy: "🔄",
        connection: "🔗",
        security: "🔒",
    };
    return icons[type] ?? "•";
};

const formatActivityTime = (timestamp: number): string => {
    const now = Date.now();
    const diff = now - timestamp;
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (seconds < 60) return `${seconds}s ago`;
    if (minutes < 60) return `${minutes}m ago`;
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;

    const date = new Date(timestamp);
    return date.toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
    });
};

const refreshActivity = () => {
    emit("refresh");
};

const clearActivity = () => {
    if (confirm("Clear all activity? This cannot be undone.")) {
        emit("clear");
    }
};

const loadMore = () => {
    displayedCount.value += props.maxDisplay;
};

const handleAction = (activity: ExtendedActivityEntry) => {
    emit("action", activity);
};
</script>

<style scoped>
.recent-activity-panel {
    padding: 20px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    background: rgba(26, 26, 46, 0.5);
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-height: 600px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
}

/* Header */
.panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.panel-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: #ffffff;
}

.header-controls {
    display: flex;
    gap: 8px;
}

.btn-icon {
    width: 32px;
    height: 32px;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: transparent;
    color: #a0a0a0;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    font-size: 14px;
}

.btn-icon:hover {
    background: rgba(0, 212, 255, 0.1);
    border-color: rgba(0, 212, 255, 0.3);
    color: #00d4ff;
}

/* Activity List */
.activity-list {
    flex: 1;
    overflow-y: auto;
    margin-right: -12px;
    padding-right: 12px;
}

.activity-list::-webkit-scrollbar {
    width: 6px;
}

.activity-list::-webkit-scrollbar-track {
    background: transparent;
}

.activity-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
}

.activity-list::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
}

.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: #606060;
}

.empty-icon {
    font-size: 32px;
    margin-bottom: 8px;
}

.empty-state p {
    margin: 0;
    font-size: 14px;
}

/* Timeline */
.activity-timeline {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.activity-entry {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 12px;
    padding: 12px;
    border-radius: 6px;
    background: rgba(0, 212, 255, 0.05);
    border: 1px solid rgba(0, 212, 255, 0.1);
    transition: all 0.2s ease;
}

.activity-entry:hover {
    background: rgba(0, 212, 255, 0.1);
    border-color: rgba(0, 212, 255, 0.2);
}

.activity-entry.error {
    background: rgba(255, 0, 110, 0.05);
    border-color: rgba(255, 0, 110, 0.1);
}

.activity-entry.error:hover {
    background: rgba(255, 0, 110, 0.1);
    border-color: rgba(255, 0, 110, 0.2);
}

.activity-entry.warning {
    background: rgba(255, 184, 0, 0.05);
    border-color: rgba(255, 184, 0, 0.1);
}

.activity-entry.warning:hover {
    background: rgba(255, 184, 0, 0.1);
    border-color: rgba(255, 184, 0, 0.2);
}

.activity-entry.success {
    background: rgba(0, 212, 255, 0.05);
    border-color: rgba(0, 212, 255, 0.1);
}

/* Timeline Dot */
.timeline-dot {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: rgba(0, 212, 255, 0.15);
    border: 1px solid rgba(0, 212, 255, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    margin-top: 2px;
}

.dot-icon {
    font-size: 14px;
}

.activity-entry.error .timeline-dot {
    background: rgba(255, 0, 110, 0.15);
    border-color: rgba(255, 0, 110, 0.3);
}

.activity-entry.warning .timeline-dot {
    background: rgba(255, 184, 0, 0.15);
    border-color: rgba(255, 184, 0, 0.3);
}

/* Activity Content */
.activity-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
}

.activity-header {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}

.activity-type-badge {
    padding: 2px 8px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    background: rgba(0, 212, 255, 0.2);
    color: #00d4ff;
    white-space: nowrap;
}

.activity-type-badge.error {
    background: rgba(255, 0, 110, 0.2);
    color: #ff006e;
}

.activity-type-badge.warning {
    background: rgba(255, 184, 0, 0.2);
    color: #ffb800;
}

.activity-type-badge.success {
    background: rgba(0, 212, 255, 0.2);
    color: #00d4ff;
}

.activity-timestamp {
    font-size: 11px;
    color: #606060;
    font-family: "Fira Code", monospace;
    margin-left: auto;
}

.activity-message {
    font-size: 13px;
    color: #ffffff;
    font-weight: 500;
    word-break: break-word;
}

.activity-details {
    font-size: 11px;
    color: #a0a0a0;
    font-family: "Fira Code", monospace;
    padding: 4px;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 3px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* Activity Action */
.activity-action {
    display: flex;
    align-items: center;
}

.btn-action {
    padding: 4px 8px;
    background: transparent;
    border: 1px solid rgba(0, 212, 255, 0.3);
    color: #00d4ff;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    text-transform: uppercase;
    letter-spacing: 0.3px;
    white-space: nowrap;
}

.btn-action:hover {
    background: rgba(0, 212, 255, 0.1);
    border-color: rgba(0, 212, 255, 0.6);
}

/* Load More */
.load-more {
    padding-top: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
}

.btn-secondary {
    width: 100%;
    padding: 10px;
    background: transparent;
    border: 1px solid rgba(0, 212, 255, 0.3);
    color: #00d4ff;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.btn-secondary:hover {
    background: rgba(0, 212, 255, 0.1);
    border-color: rgba(0, 212, 255, 0.6);
}

/* Responsive */
@media (max-width: 768px) {
    .recent-activity-panel {
        padding: 16px;
    }

    .activity-entry {
        grid-template-columns: auto 1fr;
    }

    .activity-action {
        grid-column: 2;
        justify-content: flex-start;
    }

    .activity-timestamp {
        margin-left: 0;
        order: -1;
    }
}
</style>
