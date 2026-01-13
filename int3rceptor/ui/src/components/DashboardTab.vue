<script setup lang="ts">
import { onMounted, ref } from "vue";

interface License {
    type: "Free" | "Pro" | "Enterprise";
    expiresAt: string | null;
    features: string[];
}

interface User {
    id: string;
    email: string;
    avatarUrl?: string;
}

interface Plugin {
    name: string;
    version: string;
    description: string;
    installed: boolean;
    premium: boolean;
}

const user = ref<User | null>(null);
const license = ref<License | null>(null);
const plugins = ref<Plugin[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

onMounted(async () => {
    try {
        // Fetch license info
        const licenseRes = await fetch("http://localhost:3000/api/license");
        if (licenseRes.ok) {
            const data = await licenseRes.json();

            // Map backend license to UI model
            const backendLicense = data.license;
            const tier = data.tier;

            license.value = {
                type: tier, // "Free", "Professional", "Enterprise"
                expiresAt: backendLicense?.expires_at
                    ? new Date(backendLicense.expires_at * 1000).toISOString().split('T')[0]
                    : "Never",
                features: Object.entries(data.features)
                    .filter(([_, enabled]) => enabled)
                    .map(([feature, _]) => feature.charAt(0).toUpperCase() + feature.slice(1)),
            };

            // Mock user info from license data if available, otherwise generic
            user.value = {
                id: backendLicense?.hardware_id || "unknown",
                email: backendLicense?.licensee || "Free User",
                avatarUrl: `https://api.dicebear.com/7.x/avataaars/svg?seed=${backendLicense?.licensee || 'Free'}`,
            };
        }

        // Fetch plugins
        const pluginsRes = await fetch("http://localhost:3000/api/plugins");
        if (pluginsRes.ok) {
            const data = await pluginsRes.json();
            plugins.value = data.map((p: any) => ({
                name: p.name,
                version: p.version,
                description: p.description,
                installed: p.enabled,
                premium: false, // Backend doesn't send this yet
            }));
        }
    } catch (e: any) {
        error.value = "Failed to load dashboard data: " + e.message;
        console.error(e);
    } finally {
        loading.value = false;
    }
});
</script>

<template>
    <div class="dashboard-tab">
        <div class="dashboard-header">
            <h2 class="dashboard-title">
                <span class="icon">👤</span> User Dashboard
            </h2>
        </div>

        <div v-if="loading" class="loading-state">Loading user profile...</div>

        <div v-else class="dashboard-content">
            <!-- User Profile Card -->
            <div class="card profile-card">
                <div class="profile-header">
                    <img :src="user?.avatarUrl" alt="Avatar" class="avatar" />
                    <div class="user-info">
                        <h3>{{ user?.email }}</h3>
                        <span class="user-id">ID: {{ user?.id }}</span>
                    </div>
                    <div
                        class="license-badge"
                        :class="license?.type.toLowerCase()"
                    >
                        {{ license?.type }}
                    </div>
                </div>

                <div class="license-details">
                    <div class="detail-row">
                        <span class="label">Expires:</span>
                        <span class="value">{{
                            license?.expiresAt ?? "Never"
                        }}</span>
                    </div>
                    <div class="features-list">
                        <h4>Active Features:</h4>
                        <ul>
                            <li
                                v-for="feature in license?.features"
                                :key="feature"
                            >
                                ✓ {{ feature }}
                            </li>
                        </ul>
                    </div>
                </div>
            </div>

            <!-- Plugins Section -->
            <div class="card plugins-card">
                <h3>Available Plugins</h3>
                <div class="plugins-list">
                    <div
                        v-for="plugin in plugins"
                        :key="plugin.name"
                        class="plugin-item"
                    >
                        <div class="plugin-info">
                            <div class="plugin-name-row">
                                <h4>{{ plugin.name }}</h4>
                                <span v-if="plugin.premium" class="premium-tag"
                                    >PREMIUM</span
                                >
                            </div>
                            <p>{{ plugin.description }}</p>
                            <span class="version">v{{ plugin.version }}</span>
                        </div>
                        <div class="plugin-actions">
                            <button
                                v-if="plugin.installed"
                                class="btn-installed"
                                disabled
                            >
                                Installed
                            </button>
                            <button v-else class="btn-install">Install</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.dashboard-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #0a0a0a;
    color: #ffffff;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    overflow-y: auto;
}

.dashboard-header {
    padding: 16px 20px;
    border-bottom: 1px solid #1a1a1a;
    background: linear-gradient(180deg, #0f0f0f 0%, #0a0a0a 100%);
}

.dashboard-title {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #00d4ff;
    display: flex;
    align-items: center;
    gap: 8px;
}

.loading-state {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    color: #606060;
}

.dashboard-content {
    padding: 20px;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 20px;
}

.card {
    background: #151515;
    border: 1px solid #2a2a2a;
    border-radius: 8px;
    padding: 20px;
}

.profile-header {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 20px;
    padding-bottom: 20px;
    border-bottom: 1px solid #2a2a2a;
}

.avatar {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    border: 2px solid #2a2a2a;
}

.user-info h3 {
    margin: 0 0 4px 0;
    font-size: 16px;
}

.user-id {
    font-size: 12px;
    color: #888;
}

.license-badge {
    margin-left: auto;
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
}

.license-badge.pro {
    background: rgba(0, 212, 255, 0.1);
    color: #00d4ff;
    border: 1px solid rgba(0, 212, 255, 0.3);
}

.license-badge.free {
    background: rgba(255, 255, 255, 0.1);
    color: #888;
}

.license-badge.enterprise {
    background: rgba(255, 184, 0, 0.1);
    color: #ffb800;
    border: 1px solid rgba(255, 184, 0, 0.3);
}

.detail-row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 16px;
    font-size: 14px;
}

.label {
    color: #888;
}

.features-list h4 {
    margin: 0 0 8px 0;
    font-size: 14px;
    color: #888;
}

.features-list ul {
    list-style: none;
    padding: 0;
    margin: 0;
}

.features-list li {
    font-size: 13px;
    margin-bottom: 4px;
    color: #ccc;
}

.plugins-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.plugin-item {
    background: #0f0f0f;
    border: 1px solid #2a2a2a;
    border-radius: 6px;
    padding: 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.plugin-name-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
}

.plugin-name-row h4 {
    margin: 0;
    font-size: 14px;
    color: #e0e0e0;
}

.premium-tag {
    font-size: 10px;
    background: linear-gradient(90deg, #ffb800, #ff9900);
    color: #000;
    padding: 2px 6px;
    border-radius: 4px;
    font-weight: 700;
}

.plugin-info p {
    margin: 0 0 4px 0;
    font-size: 12px;
    color: #888;
}

.version {
    font-size: 11px;
    color: #606060;
}

.btn-install {
    background: #00d4ff;
    color: #000;
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
}

.btn-install:hover {
    background: #33ddff;
}

.btn-installed {
    background: #2a2a2a;
    color: #888;
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 12px;
    cursor: default;
}
</style>
