import * as Sentry from "@sentry/browser";
import { BrowserTracing } from "@sentry/tracing";

type Tier = 0 | 1 | 2;

const DEFAULT_TRACE_RATE = 0.05;

export function initSentry() {
  const dsn = import.meta.env.VITE_SENTRY_DSN;
  if (!dsn) return;

  const tier = parseTier(import.meta.env.VITE_TELEMETRY_TIER);
  const tracesSampleRate = tier === 2 ? clampRate(import.meta.env.VITE_SENTRY_TRACES_SAMPLE_RATE ?? DEFAULT_TRACE_RATE) : 0;
  const enableTracing = tier === 2 && tracesSampleRate > 0;

  const integrations = [];
  if (enableTracing) {
    integrations.push(
      new BrowserTracing({
        tracePropagationTargets: [],
      })
    );
  }

  Sentry.init({
    dsn,
    environment: import.meta.env.VITE_SENTRY_ENVIRONMENT ?? import.meta.env.MODE,
    release: import.meta.env.VITE_SENTRY_RELEASE ?? import.meta.env.VITE_APP_VERSION ?? "dev",

    sendDefaultPii: false,
    tracesSampleRate,
    integrations,

    beforeSend(event) {
      if (event.request) {
        event.request.url = "[REDACTED_URL]";
        event.request.headers = { redacted: "true" } as any;
        event.request.cookies = "[REDACTED]";
        (event.request as any).data = undefined;
      }
      if (event.extra) {
        Object.keys(event.extra).forEach((key) => {
          if (isSensitiveKey(key)) delete (event.extra as any)[key];
        });
      }
      return event;
    },

    beforeBreadcrumb(bc) {
      if (tier === 0) return null;
      if (bc.category === "xhr" || bc.category === "fetch") return null;
      return bc;
    },
  });

  Sentry.configureScope((scope) => {
    scope.setTag("component", "ui");
    scope.setTag("tier", String(tier));
    const platform = typeof navigator !== "undefined" ? navigator.platform : "unknown";
    scope.setTag("os", platform);
  });
}

function parseTier(raw: unknown): Tier {
  const n = Number(raw);
  if (n === 1 || n === 2) return n;
  return 0;
}

function clampRate(raw: unknown): number {
  const n = typeof raw === "string" ? Number(raw) : (raw as number);
  if (!Number.isFinite(n) || n < 0) return 0;
  return Math.min(n, DEFAULT_TRACE_RATE);
}

function isSensitiveKey(key: string): boolean {
  const k = key.toLowerCase();
  return (
    k.includes("header") ||
    k.includes("cookie") ||
    k.includes("token") ||
    k.includes("auth") ||
    k.includes("payload") ||
    k.includes("body") ||
    k.includes("url")
  );
}
