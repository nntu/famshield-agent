/**
 * FamShield Cloudflare Worker - Zero-Cost Version Checker & Release Distributor
 * Single Domain: famshield.ttcantho.io.vn (Zone: ttcantho.io.vn)
 * Storage: Cloudflare KV (Metadata) + GitHub Releases (Binaries & Setup .exe)
 * Note: Only intercepts /version.json, /v1/release/*, /download/*; all other paths pass through to origin server.
 */

export interface Env {
  FAMSHIELD_VERSION_STORE?: KVNamespace;
  VERSION_STORE?: KVNamespace;
  ENVIRONMENT?: string;
  GITHUB_REPO?: string;
  ADMIN_API_KEY?: string;
}

export interface ReleaseMetadata {
  version: string;
  release_date: string;
  channel: string;
  min_supported_version: string;
  changelog: string[];
  installer_url: string;
  service_binary_url: string;
  service_sha256: string;
  signature_hex?: string;
}

const DEFAULT_METADATA: ReleaseMetadata = {
  version: "1.0.3",
  release_date: "2026-10-21",
  channel: "stable",
  min_supported_version: "1.0.3",
  changelog: [
    "Bản phát hành chính thức FamShield Agent v1.0.0",
    "Tự động tắt và dọn dẹp service cũ khi cài đặt",
    "Chụp màn hình sắc nét làm chứng cứ khi vi phạm từ khóa hoặc web độc hại",
    "Bảo vệ 100% chế độ Offline Standalone không phụ thuộc mạng"
  ],
  installer_url: "https://github.com/phamngocnghe/famshield/releases/download/v1.0.0/FamShield-Setup-v1.0.0.exe",
  service_binary_url: "https://github.com/phamngocnghe/famshield/releases/download/v1.0.0/famshield-service.exe",
  service_sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
};

const CORS_HEADERS: Record<string, string> = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, HEAD, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type, Authorization, X-Admin-Key"
};

function getKV(env: Env): KVNamespace | undefined {
  return env.FAMSHIELD_VERSION_STORE || env.VERSION_STORE;
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    const method = request.method.toUpperCase();
    const kv = getKV(env);

    // 1. Handle CORS Preflight
    if (method === "OPTIONS") {
      return new Response(null, {
        status: 204,
        headers: CORS_HEADERS
      });
    }

    // 2. Health check endpoint
    if (url.pathname === "/health" || url.pathname === "/api/health") {
      return jsonResponse({
        status: "ok",
        service: "famshield-version-checker",
        zone: "ttcantho.io.vn",
        time: new Date().toISOString()
      });
    }

    // 3. Version metadata endpoints: GET /version.json, GET /v1/release/latest
    if (
      url.pathname === "/version.json" ||
      url.pathname === "/v1/release/latest"
    ) {
      if (method !== "GET" && method !== "HEAD") {
        return errorResponse("Method not allowed", 405);
      }

      let release = DEFAULT_METADATA;

      // Fetch from KV if available
      if (kv) {
        try {
          const stored = await kv.get("latest_release", { type: "json" });
          if (stored) {
            release = stored as ReleaseMetadata;
          }
        } catch (e) {
          console.error("Failed to read from KV, falling back to default metadata:", e);
        }
      }

      return new Response(JSON.stringify(release, null, 2), {
        status: 200,
        headers: {
          ...CORS_HEADERS,
          "Content-Type": "application/json; charset=utf-8",
          // Edge cache 5 minutes, browser cache 1 minute
          "Cache-Control": "public, max-age=60, s-maxage=300"
        }
      });
    }

    // 4. Convenience redirect to latest installer binary: GET /download/latest
    if (url.pathname === "/download/latest" || url.pathname === "/v1/release/download") {
      let release = DEFAULT_METADATA;
      if (kv) {
        const stored = await kv.get("latest_release", { type: "json" });
        if (stored) {
          release = stored as ReleaseMetadata;
        }
      }
      return Response.redirect(release.installer_url, 302);
    }

    // 5. Admin API to publish new release: POST /v1/release/publish (or /v1/admin/release)
    if (url.pathname === "/v1/release/publish" || url.pathname === "/v1/admin/release") {
      if (method !== "POST") {
        return errorResponse("Method not allowed", 405);
      }

      // Check admin key
      const adminKey = request.headers.get("X-Admin-Key");
      const configuredKey = env.ADMIN_API_KEY || "famshield-default-admin-secret";
      if (!adminKey || adminKey !== configuredKey) {
        return errorResponse("Unauthorized: Invalid X-Admin-Key", 401);
      }

      try {
        const body = (await request.json()) as Partial<ReleaseMetadata>;
        if (!body.version || !body.installer_url) {
          return errorResponse("Missing required fields: version and installer_url", 400);
        }

        const newRelease: ReleaseMetadata = {
          version: body.version,
          release_date: body.release_date || new Date().toISOString().split("T")[0],
          channel: body.channel || "stable",
          min_supported_version: body.min_supported_version || "1.0.0",
          changelog: Array.isArray(body.changelog) ? body.changelog : [],
          installer_url: body.installer_url,
          service_binary_url: body.service_binary_url || body.installer_url,
          service_sha256: body.service_sha256 || "",
          signature_hex: body.signature_hex
        };

        if (kv) {
          await kv.put("latest_release", JSON.stringify(newRelease));
          // Also archive version under key `release:vX.Y.Z`
          await kv.put(`release:${newRelease.version}`, JSON.stringify(newRelease));
        }

        return jsonResponse({
          success: true,
          message: `Successfully published release v${newRelease.version}`,
          release: newRelease
        });
      } catch (err: any) {
        return errorResponse(`Failed to process release payload: ${err.message}`, 400);
      }
    }

    // 6. Fallback: If not an update/release route, pass through to the origin server (famshield.ttcantho.io.vn)
    try {
      return await fetch(request);
    } catch {
      return errorResponse("Not Found", 404);
    }
  }
};

function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data, null, 2), {
    status,
    headers: {
      ...CORS_HEADERS,
      "Content-Type": "application/json; charset=utf-8"
    }
  });
}

function errorResponse(message: string, status = 400): Response {
  return new Response(JSON.stringify({ error: message, status }, null, 2), {
    status,
    headers: {
      ...CORS_HEADERS,
      "Content-Type": "application/json; charset=utf-8"
    }
  });
}
