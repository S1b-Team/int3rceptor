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

// ==========================================
// NOWARU - Traffic Intelligence Types
// ==========================================

export interface ParsedHeader {
    name: string;
    value: string;
}

export interface ParsedRequest {
    id: number;
    timestamp: string;
    method: string;
    url: string;
    path: string;
    query?: string;
    query_params: Record<string, string>;
    host: string;
    http_version: string;
    headers: ParsedHeader[];
    content_type?: string;
    content_category: ContentCategory;
    body_text?: string;
    body_raw: number[];
    body_size: number;
    tls: boolean;
    cookies: Record<string, string>;
}

export interface ParsedResponse {
    request_id: number;
    status_code: number;
    status_text: string;
    status_category: StatusCategory;
    http_version: string;
    headers: ParsedHeader[];
    content_type?: string;
    content_category: ContentCategory;
    body_text?: string;
    body_raw: number[];
    body_size: number;
    duration_ms: number;
    set_cookies: string[];
}

export interface HttpTransaction {
    id: number;
    request: ParsedRequest;
    response?: ParsedResponse;
    tags: string[];
    notes?: string;
    highlight?: string;
}

export type ContentCategory =
    | "json"
    | "xml"
    | "html"
    | "javascript"
    | "css"
    | "form_data"
    | "multipart"
    | "text"
    | "binary"
    | "unknown";

export type StatusCategory =
    | "informational"
    | "success"
    | "redirect"
    | "client_error"
    | "server_error";

export interface SyntaxToken {
    type: string;
    content: string;
    line: number;
    column: number;
}

export interface HighlightedContent {
    raw: string;
    tokens: SyntaxToken[];
    category: ContentCategory;
    line_count: number;
}

export interface TransactionFilter {
    method?: string;
    host?: string;
    path_contains?: string;
    status_category?: StatusCategory;
    content_category?: ContentCategory;
    tls?: boolean;
    has_response?: boolean;
    tags?: string[];
    search?: string;
    limit?: number;
    offset?: number;
}

export interface TrafficStats {
    total_transactions: number;
    requests_by_method: Record<string, number>;
    responses_by_status: Record<number, number>;
    content_types: Record<string, number>;
    total_bytes_sent: number;
    total_bytes_received: number;
    avg_response_time_ms: number;
}

// ==========================================
// VOIDWALKER - WebSocket Analysis Types
// ==========================================

export interface ConnectionSummary {
    id: string;
    http_request_id: number;
    url: string;
    host: string;
    protocol?: string;
    state: ConnectionState;
    frames_sent: number;
    frames_received: number;
    bytes_sent: number;
    bytes_received: number;
    started_at: string;
    ended_at?: string;
    secure: boolean;
}

export type ConnectionState =
    | "connecting"
    | "open"
    | "closing"
    | "closed"
    | "failed";

export type FrameDirection = "sent" | "received";

export type FrameType =
    | "text"
    | "binary"
    | "close"
    | "ping"
    | "pong"
    | "continuation";

export interface WebSocketFrame {
    id: number;
    connection_id: string;
    direction: FrameDirection;
    frame_type: FrameType;
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

export interface FrameFilter {
    connection_id?: string;
    direction?: FrameDirection;
    frame_type?: FrameType;
    search?: string;
    min_size?: number;
    max_size?: number;
    limit?: number;
    offset?: number;
}

export interface FrameDiff {
    frame_a_id: number;
    frame_b_id: number;
    changes: DiffChange[];
    original: string;
    modified: string;
    insertions: number;
    deletions: number;
    similarity: number;
}

export interface DiffChange {
    change_type: "equal" | "insert" | "delete";
    content: string;
    old_line?: number;
    new_line?: number;
}

export interface WebSocketStats {
    active_connections: number;
    total_connections: number;
    total_frames: number;
    text_frames: number;
    binary_frames: number;
    control_frames: number;
    total_bytes: number;
    frames_by_connection: Record<string, number>;
}

export interface ReplayMessage {
    frame_id: number;
    target_connection?: string;
    content: string;
    frame_type: FrameType;
}
