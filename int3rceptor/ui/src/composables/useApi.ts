import type {
    CaptureEntry,
    HeaderField,
    IntruderGenerateRequest,
    IntruderGenerateResponse,
    IntruderResult,
    ReplayResult,
    Rule,
    ScopeConfig,
} from "@/types";
import axios from "axios";

const api = axios.create({
    baseURL: import.meta.env.VITE_API_URL ?? "http://127.0.0.1:3000",
});

export function useApi() {
    const listRequests = async (
        filters: Record<string, any> = {}
    ): Promise<CaptureEntry[]> => {
        const res = await api.get<CaptureEntry[]>("/api/requests", {
            params: filters,
        });
        return res.data;
    };

    const clearRequests = async (): Promise<void> => {
        await api.delete("/api/requests");
    };

    const repeatRequest = async (
        id: number,
        payload: {
            method: string;
            url: string;
            headers: HeaderField[];
            body: string;
        }
    ): Promise<ReplayResult> => {
        const res = await api.post<ReplayResult>(`/api/requests/${id}/repeat`, {
            id,
            method: payload.method,
            url: payload.url,
            headers: payload.headers,
            modified_body: payload.body,
        });
        return res.data;
    };

    const exportRequests = async (
        format: "json" | "csv" | "har",
        filters: Record<string, any> = {}
    ): Promise<Blob> => {
        const res = await api.get(`/api/requests/export`, {
            params: { ...filters, format },
            responseType: "blob",
        });
        return res.data;
    };

    const listRules = async (): Promise<Rule[]> => {
        const res = await api.get<Rule[]>("/api/rules");
        return res.data;
    };

    const addRule = async (rule: Rule): Promise<void> => {
        await api.post("/api/rules", rule);
    };

    const clearRules = async (): Promise<void> => {
        await api.delete("/api/rules");
    };

    const getScope = async (): Promise<ScopeConfig> => {
        const { data } = await api.get<ScopeConfig>("/api/scope");
        return data;
    };

    const setScope = async (config: ScopeConfig) => {
        await api.put("/api/scope", config);
    };

    const intruderGenerate = async (
        request: IntruderGenerateRequest
    ): Promise<IntruderGenerateResponse> => {
        const { data } = await api.post<IntruderGenerateResponse>(
            "/api/intruder/generate",
            request
        );
        return data;
    };

    const intruderResults = async (): Promise<IntruderResult[]> => {
        const { data } = await api.get<IntruderResult[]>(
            "/api/intruder/results"
        );
        return data;
    };

    const intruderClear = async (): Promise<void> => {
        await api.delete("/api/intruder/results");
    };

    const intruderStart = async (
        request: IntruderGenerateRequest
    ): Promise<void> => {
        await api.post("/api/intruder/start", request);
    };

    const intruderStop = async (): Promise<void> => {
        await api.post("/api/intruder/stop");
    };

    return {
        listRequests,
        clearRequests,
        repeatRequest,
        exportRequests,
        listRules,
        addRule,
        clearRules,
        getScope,
        setScope,
        intruderGenerate,
        intruderResults,
        intruderClear,
        intruderStart,
        intruderStop,
    };
}

export { api };
