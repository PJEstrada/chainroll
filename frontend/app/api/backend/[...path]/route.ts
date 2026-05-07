import type { NextRequest } from "next/server";

const BACKEND_URL_HEADER = "x-boto-backend-url";
const DEFAULT_BACKEND_URL = process.env.BOTO_API_URL ?? "http://localhost:3001";
const FORWARDED_HEADERS = ["x-tenant-id", "x-actor-id", "content-type"];

type RouteContext = {
  params: Promise<{
    path?: string[];
  }>;
};

function backendUrlFor(request: NextRequest, path: string[]) {
  const rawBase =
    request.headers.get(BACKEND_URL_HEADER) ?? DEFAULT_BACKEND_URL;
  const base = rawBase.replace(/\/$/, "");
  const url = new URL(`${base}/${path.join("/")}`);
  url.search = request.nextUrl.search;
  return url;
}

function proxyHeaders(request: NextRequest) {
  const headers = new Headers();

  for (const name of FORWARDED_HEADERS) {
    const value = request.headers.get(name);
    if (value) {
      headers.set(name, value);
    }
  }

  return headers;
}

async function proxy(request: NextRequest, context: RouteContext) {
  const { path = [] } = await context.params;
  const method = request.method;
  const hasBody = !["GET", "HEAD"].includes(method);

  try {
    const response = await fetch(backendUrlFor(request, path), {
      method,
      headers: proxyHeaders(request),
      body: hasBody ? await request.text() : undefined,
      cache: "no-store",
    });

    const headers = new Headers();
    const contentType = response.headers.get("content-type");
    if (contentType) {
      headers.set("content-type", contentType);
    }

    return new Response(await response.arrayBuffer(), {
      status: response.status,
      headers,
    });
  } catch {
    return Response.json(
      {
        error:
          "Unable to reach the Boto API. Check the backend URL and make sure the Rust API is running.",
      },
      { status: 502 },
    );
  }
}

export const GET = proxy;
export const POST = proxy;
export const PUT = proxy;
export const DELETE = proxy;
