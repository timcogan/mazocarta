const CACHE_VERSION = "__MAZOCARTA_SW_VERSION__";
const APP_SCOPE_URL = new URL("./", self.location.href);
const APP_SCOPE = APP_SCOPE_URL.toString();
const INDEX_URL = new URL("./index.html", self.location.href).toString();
const SCOPE_PATH = APP_SCOPE_URL.pathname;
const IS_PREVIEW_SCOPE = SCOPE_PATH.endsWith("/preview/");
const CACHE_CHANNEL = IS_PREVIEW_SCOPE ? "preview" : "root";
const CACHE_PREFIX = `mazocarta-shell-${CACHE_CHANNEL}-`;
const SHELL_CACHE = `${CACHE_PREFIX}${CACHE_VERSION}`;
const PREVIEW_PATH = IS_PREVIEW_SCOPE ? null : new URL("preview/", APP_SCOPE_URL).pathname;
const PREVIEW_PATH_WITHOUT_SLASH =
  PREVIEW_PATH && PREVIEW_PATH.endsWith("/") ? PREVIEW_PATH.slice(0, -1) : PREVIEW_PATH;
const CRITICAL_PRECACHE_URLS = [
  APP_SCOPE,
  INDEX_URL,
  new URL("./index.js", self.location.href).toString(),
  new URL("./mazocarta.wasm", self.location.href).toString(),
];
const OPTIONAL_PRECACHE_URLS = [
  new URL("./multiplayer.js", self.location.href).toString(),
  new URL("./styles.css", self.location.href).toString(),
  new URL("./ui-kit.css", self.location.href).toString(),
  new URL("./ui-overlays.css", self.location.href).toString(),
  new URL("./ui-kit.js", self.location.href).toString(),
  new URL("./jsqr.js", self.location.href).toString(),
  new URL("./qrcode.bundle.mjs", self.location.href).toString(),
  new URL("./mazocarta.svg", self.location.href).toString(),
  new URL("./icons/combat/heart.png", self.location.href).toString(),
  new URL("./icons/combat/shield.png", self.location.href).toString(),
  new URL("./icons/combat/energy.png", self.location.href).toString(),
  new URL("./icons/combat/deck.png", self.location.href).toString(),
  new URL("./icons/combat/arrow.png", self.location.href).toString(),
  new URL("./manifest.webmanifest?v=3", self.location.href).toString(),
  new URL("./apple-touch-icon.png?v=3", self.location.href).toString(),
  new URL("./icons/icon-any-192.png?v=3", self.location.href).toString(),
  new URL("./icons/icon-any-512.png?v=3", self.location.href).toString(),
  new URL("./icons/icon-maskable-192.png?v=3", self.location.href).toString(),
  new URL("./icons/icon-maskable-512.png?v=3", self.location.href).toString(),
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(SHELL_CACHE)
      .then(async (cache) => {
        await cache.addAll(
          CRITICAL_PRECACHE_URLS.map((url) => new Request(url, { cache: "reload" })),
        );
        await Promise.allSettled(
          OPTIONAL_PRECACHE_URLS.map(async (url) => {
            const request = new Request(url, { cache: "reload" });
            const response = await fetch(request);
            if (response.ok) {
              await cache.put(request, response);
            }
          }),
        );
      }),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key.startsWith(CACHE_PREFIX) && key !== SHELL_CACHE)
            .map((key) => caches.delete(key)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("message", (event) => {
  if (event.data?.type === "SKIP_WAITING") {
    void self.skipWaiting();
  }
});

function shouldBypass(url) {
  // Keep the root service worker from hijacking the preview subtree.
  const targetsPreview =
    PREVIEW_PATH != null &&
    (url.pathname === PREVIEW_PATH_WITHOUT_SLASH || url.pathname.startsWith(PREVIEW_PATH));

  return (
    url.origin !== self.location.origin ||
    url.pathname.endsWith("/.debug-mode.json") ||
    url.pathname.endsWith("/sw.js") ||
    targetsPreview
  );
}

async function staleWhileRevalidate(request, fallbackUrl) {
  const cache = await caches.open(SHELL_CACHE);
  const cached = await cache.match(request);
  const networkPromise = fetch(request)
    .then((response) => {
      if (response.ok) {
        cache.put(request, response.clone());
      }
      return response;
    })
    .catch(() => null);

  if (cached) {
    void networkPromise;
    return cached;
  }

  const networkResponse = await networkPromise;
  if (networkResponse) {
    return networkResponse;
  }

  if (fallbackUrl) {
    const fallback = await cache.match(fallbackUrl);
    if (fallback) {
      return fallback;
    }
  }

  return new Response("Offline", {
    status: 503,
    statusText: "Offline",
    headers: {
      "Content-Type": "text/plain; charset=utf-8",
    },
  });
}

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET") {
    return;
  }

  const url = new URL(request.url);
  if (shouldBypass(url)) {
    return;
  }

  if (request.mode === "navigate") {
    event.respondWith(staleWhileRevalidate(request, INDEX_URL));
    return;
  }

  event.respondWith(staleWhileRevalidate(request));
});
