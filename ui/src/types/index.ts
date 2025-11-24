export interface CapturedRequest {
    id: number;
    timestamp_ms: number;
    method: string;
    url: string;
    headers: [string, string][];
    body: number[];
    tls: boolean;
}

export interface CapturedResponse {
    request_id: number;
    status_code: number;
    headers: [string, string][];
    body: number[];
    duration_ms: number;
}

export interface CaptureEntry {
    request: CapturedRequest;
    response?: CapturedResponse | null;
}

export interface HeaderField {
    name: string;
    value: string;
}

export interface ReplayResult {
    id: number;
    status: number;
    duration_ms: number;
    timestamp_ms: number;
    headers: [string, string][];
    body_preview: string;
}

export type RuleType = "Request" | "Response";

export interface ScopeConfig {
    includes: string[];
    excludes: string[];
}

export type AttackType = "Sniper" | "Battering" | "Pitchfork" | "ClusterBomb";

export interface IntruderPosition {
    start: number;
    end: number;
    name: string;
}

export interface IntruderConfig {
    positions: IntruderPosition[];
    payloads: string[];
    attack_type: AttackType;
}

export interface IntruderResult {
    request_id: number;
    payload: string;
    status_code: number;
    response_length: number;
    duration_ms: number;
}

export interface IntruderGenerateRequest {
    template: string;
    config: IntruderConfig;
}

export interface IntruderGenerateResponse {
    requests: string[];
}
export type MatchCondition =
    // Simple substring matching
    | { UrlContains: string }
    | { HeaderContains: [string, string] }
    | { BodyContains: string }
    // Advanced regex matching
    | { UrlRegex: string }
    | { HeaderRegex: [string, string] }
    | { BodyRegex: string };

export type Action =
    // Simple replacements
    | { ReplaceBody: [string, string] }
    | { SetHeader: [string, string] }
    | { RemoveHeader: string }
    // Advanced regex replacements
    | { RegexReplaceBody: [string, string] }
    | { RegexReplaceHeader: [string, string, string] };

export interface Rule {
    id: string;
    active: boolean;
    rule_type: RuleType;
    condition: MatchCondition;
    action: Action;
}
