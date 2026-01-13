import axios, { type AxiosInstance } from "axios";

// INT3RCEPTOR Backend API Client
const API_BASE_URL = "http://localhost:3000/api";

class INT3RCEPTORClient {
    private client: AxiosInstance;
    public readonly baseUrl: string;

    constructor(baseURL: string = API_BASE_URL) {
        this.baseUrl = baseURL;
        this.client = axios.create({
            baseURL,
            timeout: 10000,
            headers: {
                "Content-Type": "application/json",
            },
        });

        // Response interceptor for error handling
        this.client.interceptors.response.use(
            (response) => response,
            (error) => {
                console.error("API Error:", error.message);
                return Promise.reject(error);
            }
        );
    }

    // Traffic API
    async getTraffic(limit = 100) {
        try {
            const response = await this.client.get("/traffic", {
                params: { limit },
            });
            return response.data;
        } catch (error) {
            console.error("Failed to fetch traffic:", error);
            return [];
        }
    }

    async getRequest(id: string) {
        const response = await this.client.get(`/traffic/${id}`);
        return response.data;
    }

    async clearTraffic() {
        const response = await this.client.delete("/traffic");
        return response.data;
    }

    // Repeater API
    async sendRepeaterRequest(request: {
        method: string;
        url: string;
        headers?: Record<string, string>;
        body?: string;
    }) {
        const response = await this.client.post("/repeater/send", request);
        return response.data;
    }

    // Settings API
    async getSettings() {
        const response = await this.client.get("/settings");
        return response.data;
    }

    async updateSettings(settings: any) {
        const response = await this.client.put("/settings", settings);
        return response.data;
    }

    // Plugins API
    async getPlugins() {
        try {
            const response = await this.client.get("/plugins");
            return response.data;
        } catch (error) {
            console.error("Failed to fetch plugins:", error);
            return [];
        }
    }

    async reloadPlugin(name: string) {
        const response = await this.client.post(`/plugins/${name}/reload`);
        return response.data;
    }

    async togglePlugin(name: string, enabled: boolean) {
        const response = await this.client.post(`/plugins/${name}/toggle`, {
            enabled,
        });
        return response.data;
    }

    async uploadPlugin(file: File) {
        const formData = new FormData();
        formData.append('plugin', file);
        const response = await this.client.post('/plugins/upload', formData, {
            headers: {
                'Content-Type': 'multipart/form-data',
            },
        });
        return response.data;
    }

    // Stats API
    async getStats() {
        try {
            const response = await this.client.get("/stats");
            return response.data;
        } catch (error) {
            console.error("Failed to fetch stats:", error);
            return {
                requests: 0,
                memory: 0,
                connections: 0,
                uptime: 0,
            };
        }
    }

    // WebSocket API (for real-time updates)
    async getWebSocketStats() {
        try {
            const response = await this.client.get("/websocket/stats");
            return response.data;
        } catch (error) {
            console.error("Failed to fetch WebSocket stats:", error);
            return {
                connections: 0,
                frames: 0,
            };
        }
    }

    // Rules API
    async getRules() {
        const response = await this.client.get("/rules");
        return response.data;
    }

    async createRule(rule: any) {
        const response = await this.client.post("/rules", rule);
        return response.data;
    }

    async deleteRule(id: string) {
        const response = await this.client.delete(`/rules/${id}`);
        return response.data;
    }

    // Scope API
    async getScope() {
        const response = await this.client.get("/scope");
        return response.data;
    }

    async updateScope(scope: any) {
        const response = await this.client.put("/scope", scope);
        return response.data;
    }

    // Intruder API
    async intruderGenerate(request: any) {
        const response = await this.client.post("/intruder/generate", request);
        return response.data;
    }

    async intruderStart(request: any) {
        const response = await this.client.post("/intruder/start", request);
        return response.data;
    }

    async intruderStop() {
        const response = await this.client.post("/intruder/stop");
        return response.data;
    }

    async intruderResults() {
        const response = await this.client.get("/intruder/results");
        return response.data;
    }

    async intruderClear() {
        const response = await this.client.delete("/intruder/results");
        return response.data;
    }

    // Scanner API
    async scannerGetConfig() {
        const response = await this.client.get("/scanner/config");
        return response.data;
    }

    async scannerSetConfig(config: ScanConfig) {
        const response = await this.client.put("/scanner/config", config);
        return response.data;
    }

    async scannerGetRules() {
        const response = await this.client.get("/scanner/rules");
        return response.data;
    }

    async scannerStart(targets: string[]) {
        const response = await this.client.post("/scanner/start", { targets });
        return response.data;
    }

    async scannerStop() {
        const response = await this.client.post("/scanner/stop");
        return response.data;
    }

    async scannerGetFindings() {
        const response = await this.client.get("/scanner/findings");
        return response.data;
    }

    async scannerClearFindings() {
        const response = await this.client.delete("/scanner/findings");
        return response.data;
    }

    async scannerGetStats(): Promise<ScanStats> {
        const response = await this.client.get("/scanner/stats");
        return response.data;
    }

    // Health check
    async healthCheck() {
        try {
            const response = await this.client.get("/health");
            return response.status === 200;
        } catch {
            return false;
        }
    }

    // Encoding API
    async encodingTransform(
        text: string,
        encoding: EncodingType,
        operation: TransformOperation
    ) {
        const response = await this.client.post("/encoding/transform", {
            text,
            encoding,
            operation,
        });
        return response.data;
    }

    // Comparer API
    async comparerDiff(left: string, right: string, mode: CompareMode) {
        const response = await this.client.post("/comparer/diff", {
            left,
            right,
            mode,
        });
        return response.data;
    }

    // WebSocket API
    async wsGetConnections(): Promise<WsConnection[]> {
        const response = await this.client.get("/websocket/connections");
        return response.data;
    }

    async wsGetFrames(connectionId: string): Promise<WsFrame[]> {
        const response = await this.client.get(`/websocket/frames/${connectionId}`);
        return response.data;
    }

    async wsClear() {
        const response = await this.client.delete("/websocket/clear");
        return response.data;
    }

    // Project API
    async projectSave(path: string, scope: string[]) {
        const response = await this.client.post("/project/save", { path, scope });
        return response.data;
    }

    async projectLoad(path: string) {
        const response = await this.client.post("/project/load", { path });
        return response.data;
    }

    async projectInfo() {
        const response = await this.client.get("/project/info");
        return response.data;
    }

    async projectUpdate(name?: string, description?: string) {
        const response = await this.client.put("/project/info", { name, description });
        return response.data;
    }

    async projectNew(name: string) {
        const response = await this.client.post("/project/new", { name });
        return response.data;
    }
}

// Export singleton instance
export const apiClient = new INT3RCEPTORClient();

// Export types
export type EncodingType = "base64" | "url" | "hex" | "html" | "rot13";
export type TransformOperation = "encode" | "decode";
export type CompareMode = "lines" | "words" | "chars";

export interface TransformResponse {
    text: string;
    error?: string;
}

export interface DiffChange {
    tag: "equal" | "delete" | "insert";
    value: string;
}

export interface CompareResponse {
    changes: DiffChange[];
}
export interface TrafficItem {
    id: string;
    method: string;
    url: string;
    status: number;
    size: number;
    time: number;
    timestamp: string;
}

export interface PluginInfo {
    name: string;
    version: string;
    enabled: boolean;
    description: string;
}

export interface Stats {
    requests: number;
    memory: number;
    connections: number;
    uptime: number;
}

export type AttackType = "Sniper" | "Battering" | "Pitchfork" | "ClusterBomb";

export interface IntruderPosition {
    start: number;
    end: number;
    name: string;
}

export interface IntruderOptions {
    concurrency: number;
    delay_ms: number;
}

export interface IntruderConfig {
    positions: IntruderPosition[];
    payloads: string[];
    attack_type: AttackType;
    options: IntruderOptions;
}

export interface IntruderResult {
    request_id: number;
    payload: string;
    status_code: number;
    response_length: number;
    duration_ms: number;
}

// Scanner Types
export type Severity = "critical" | "high" | "medium" | "low" | "info";

export type VulnerabilityCategory =
    | "injection"
    | "xss"
    | "broken_auth"
    | "sensitive_data_exposure"
    | "xxe"
    | "broken_access_control"
    | "security_misconfiguration"
    | "csrf"
    | "vulnerable_components"
    | "insufficient_logging"
    | "ssrf"
    | "path_traversal"
    | "information_disclosure"
    | "open_redirect"
    | "other";

export interface ScanConfig {
    passive: boolean;
    active: boolean;
    categories: VulnerabilityCategory[];
    concurrency: number;
    delay_ms: number;
    follow_redirects: boolean;
    max_depth: number;
}

export interface Finding {
    id: string;
    rule_id: string;
    category: VulnerabilityCategory;
    severity: Severity;
    title: string;
    description: string;
    url: string;
    evidence: string;
    remediation: string;
    references: string[];
    timestamp: string;
    confirmed: boolean;
}

export interface ScanStats {
    is_running: boolean;
    requests_scanned: number;
    vulnerabilities_found: number;
    critical_count: number;
    high_count: number;
    medium_count: number;
    low_count: number;
    info_count: number;
}

// WebSocket Types
export interface WsConnection {
    id: string;
    url: string;
    established_at: number;
    closed_at: number | null;
    frames_count: number;
}

export type WsFrameType = "Text" | "Binary" | "Ping" | "Pong" | "Close";
export type WsDirection = "ClientToServer" | "ServerToClient";

export interface WsFrame {
    id: number;
    connection_id: string;
    timestamp: number;
    direction: WsDirection;
    frame_type: WsFrameType;
    payload: number[];
    masked: boolean;
}
