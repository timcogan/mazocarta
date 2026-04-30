import { createMultiplayerController } from "./multiplayer.js";

const canvas = document.getElementById("game");
if (!(canvas instanceof HTMLCanvasElement)) {
  throw new Error("Canvas element `#game` not found");
}
const playerNameInput = document.getElementById("player-name-input");
if (!(playerNameInput instanceof HTMLInputElement)) {
  throw new Error("Input element `#player-name-input` not found");
}
const playerNameOverlay = document.getElementById("player-name-overlay");
const playerNameOverlayPrefix = document.getElementById("player-name-overlay-prefix");
const playerNameOverlayCaret = document.getElementById("player-name-overlay-caret");
const playerNameOverlaySuffix = document.getElementById("player-name-overlay-suffix");
const typingPointerOverlay = document.getElementById("typing-pointer-overlay");
if (!(playerNameOverlay instanceof HTMLDivElement)) {
  throw new Error("Overlay element `#player-name-overlay` not found");
}
if (!(playerNameOverlayPrefix instanceof HTMLSpanElement)) {
  throw new Error("Overlay element `#player-name-overlay-prefix` not found");
}
if (!(playerNameOverlayCaret instanceof HTMLSpanElement)) {
  throw new Error("Overlay element `#player-name-overlay-caret` not found");
}
if (!(playerNameOverlaySuffix instanceof HTMLSpanElement)) {
  throw new Error("Overlay element `#player-name-overlay-suffix` not found");
}
if (!(typingPointerOverlay instanceof HTMLDivElement)) {
  throw new Error("Overlay element `#typing-pointer-overlay` not found");
}
const ctx = canvas.getContext("2d");
if (!ctx) {
  throw new Error("2D canvas context unavailable.");
}
const decoder = new TextDecoder();
const encoder = new TextEncoder();
const SEARCH_PARAMS = new URLSearchParams(window.location.search);
const E2E_MODE = SEARCH_PARAMS.get("e2e") === "1";
const blurCanvas = document.createElement("canvas");
const blurCtx = blurCanvas.getContext("2d");
const imageCache = new Map();
const tintedImageCache = new Map();
const spriteCache = new Map();
const spriteColorCache = new Map();
const spriteColorProbeCanvas = document.createElement("canvas");
spriteColorProbeCanvas.width = 1;
spriteColorProbeCanvas.height = 1;
const spriteColorProbeCtx = spriteColorProbeCanvas.getContext("2d");
const spriteStyleByColor = new Map([
  ["#8dffad", { tier: 1, variant: "base" }],
  ["#efff6f", { tier: 1, variant: "detailA" }],
  ["#39e8ff", { tier: 1, variant: "detailB" }],
  ["#7fb6ff", { tier: 1, variant: "detailC" }],
  ["#1fba63", { tier: 1, variant: "detailD" }],
  ["#ffe39a", { tier: 1, variant: "detailE" }],
  ["#6f9f7b", { tier: 1, variant: "dim" }],
  ["#c7a7ff", { tier: 2, variant: "base" }],
  ["#ff9df3", { tier: 2, variant: "detailA" }],
  ["#7f89ff", { tier: 2, variant: "detailB" }],
  ["#79e7ff", { tier: 2, variant: "detailC" }],
  ["#b65cff", { tier: 2, variant: "detailD" }],
  ["#ffe9b8", { tier: 2, variant: "detailE" }],
  ["#7f719b", { tier: 2, variant: "dim" }],
  ["#ffb852", { tier: 3, variant: "base" }],
  ["#ffe07a", { tier: 3, variant: "detailA" }],
  ["#ff6438", { tier: 3, variant: "detailB" }],
  ["#ff4f8a", { tier: 3, variant: "detailC" }],
  ["#fff27a", { tier: 3, variant: "detailD" }],
  ["#9fe7ff", { tier: 3, variant: "detailE" }],
  ["#9a7657", { tier: 3, variant: "dim" }],
]);
const PLAYER_NAME_MAX_CHARS = 12;

let wasm = null;
let rafId = 0;
let lastFrameTime = 0;
let logicalWidth = 1280;
let logicalHeight = 720;
let lastRunSaveGeneration = 0;
let lastRunSaveRaw = null;
let lastPartySnapshotGeneration = 0;
let lastPartySnapshot = null;
let lastRunSummary = null;
let lastLanguageGeneration = 0;
let lastBackgroundModeGeneration = 0;
let lastPlayerNameGeneration = 0;
let deferredInstallPrompt = null;
let serviceWorkerRegistration = null;
let waitingServiceWorker = null;
let reloadOnNextControllerChange = false;
let lastBootScreenVisible = null;
let e2eSupport = null;
let e2eSupportPromise = Promise.resolve(null);
let playerNameInputVisible = false;
let playerNameEditingActive = false;
let lastMouseClientPoint = null;
let lastPointerDownStartedOnPlayerNameInput = false;
const MONO_STACK =
  '"IBM Plex Mono", "JetBrains Mono", "Cascadia Mono", "Fira Code", "Liberation Mono", monospace';
const TERM_GREEN = "#33ff66";
const TERM_GREEN_SOFT = "#8dffad";
const TERM_INK = "#f4e8cf";
const GAME_UI_STROKE_START = "rgba(51, 255, 102, 0.85)";
const GAME_UI_STROKE_PANEL = "rgba(51, 255, 102, 0.38)";
const GAME_UI_FILL_ALPHA = 0.04;
const GAME_UI_STROKE_ALPHA_BOOST = 0.18;
const GAME_UI_STROKE_WIDTH = 0.5;
const GAME_BUTTON_RADIUS = 8;
const GAME_BUTTON_FONT_SIZE = 28;
const GAME_BUTTON_MIN_FONT_SIZE = 18;
const GAME_BUTTON_PAD_X = 12;
const GAME_BUTTON_PAD_Y = 8;
const GAME_BUTTON_MIN_PAD_X = 8;
const GAME_BUTTON_MIN_PAD_Y = 6;
const DEFAULT_GAME_TITLE = "Mazocarta";
const PREVIEW_GAME_TITLE = "Mazocarta Preview";
const LOGO_ASSET_PATH = "./mazocarta.svg";
const COMBAT_ICON_ASSET_PATHS = [
  "./icons/combat/heart.png",
  "./icons/combat/shield.png",
  "./icons/combat/energy.png",
  "./icons/combat/deck.png",
  "./icons/combat/arrow.png",
];
const SHARE_CARD_WIDTH = 420;
const SHARE_CARD_HEIGHT = 500;

function readMetaContent(name) {
  const content = document.querySelector(`meta[name="${name}"]`)?.getAttribute("content");
  return typeof content === "string" && content.length > 0 ? content : null;
}

function resolveAppChannel() {
  const metaChannel = readMetaContent("mazocarta-app-channel");
  if (metaChannel === "preview" || metaChannel === "stable") {
    return metaChannel;
  }
  return window.location.pathname.includes("/preview/") ? "preview" : "stable";
}

const APP_CHANNEL = resolveAppChannel();
const GAME_TITLE =
  readMetaContent("mazocarta-app-title") ||
  (APP_CHANNEL === "preview" ? PREVIEW_GAME_TITLE : DEFAULT_GAME_TITLE);
const STORAGE_NAMESPACE = APP_CHANNEL === "preview" ? "mazocarta.preview" : "mazocarta";
const ACTIVE_RUN_STORAGE_KEY = `${STORAGE_NAMESPACE}.active_run`;
const LANGUAGE_STORAGE_KEY = `${STORAGE_NAMESPACE}.language`;
const BACKGROUND_MODE_STORAGE_KEY = `${STORAGE_NAMESPACE}.background_mode`;
const PLAYER_NAME_STORAGE_KEY = `${STORAGE_NAMESPACE}.player_name`;

function isStandaloneMode() {
  return (
    window.matchMedia?.("(display-mode: standalone)")?.matches ||
    window.navigator.standalone === true
  );
}

function isIosDevice() {
  return (
    /iPad|iPhone|iPod/.test(window.navigator.userAgent) ||
    (window.navigator.platform === "MacIntel" && window.navigator.maxTouchPoints > 1)
  );
}

function isSafariBrowser() {
  const userAgent = window.navigator.userAgent;
  return /Safari/i.test(userAgent) && !/CriOS|FxiOS|EdgiOS|OPiOS|Chrome|Chromium/i.test(userAgent);
}

function isLocalDevHost() {
  return (
    window.location.hostname === "localhost" ||
    window.location.hostname === "127.0.0.1" ||
    window.location.hostname === "::1"
  );
}

function isAndroidAppHost() {
  return (
    window.location.protocol === "https:" &&
    window.location.hostname === "appassets.androidplatform.net"
  );
}

function resolveInstallCapabilityCode() {
  if (isAndroidAppHost()) {
    return 0;
  }
  if (isStandaloneMode()) {
    return 3;
  }
  if (deferredInstallPrompt) {
    return 1;
  }
  if (isIosDevice() && isSafariBrowser()) {
    return 2;
  }
  return 0;
}

async function resolveDebugEnabled() {
  const value = SEARCH_PARAMS.get("debug");
  if (value === "1" || value === "true") {
    return true;
  }
  if (value === "0" || value === "false") {
    return false;
  }

  try {
    const response = await fetch("./.debug-mode.json", { cache: "no-store" });
    if (!response.ok) {
      return false;
    }
    const payload = await response.json();
    return payload?.enabled !== false;
  } catch {
    return false;
  }
}

function resizeCanvas() {
  const dpr = Math.max(1, window.devicePixelRatio || 1);
  const rect = canvas.getBoundingClientRect();
  const nextLogicalWidth = Math.max(1, Math.round(rect.width));
  const nextLogicalHeight = Math.max(1, Math.round(rect.height));
  const width = Math.max(1, Math.round(nextLogicalWidth * dpr));
  const height = Math.max(1, Math.round(nextLogicalHeight * dpr));

  if (
    canvas.width !== width ||
    canvas.height !== height ||
    logicalWidth !== nextLogicalWidth ||
    logicalHeight !== nextLogicalHeight
  ) {
    canvas.width = width;
    canvas.height = height;
    logicalWidth = nextLogicalWidth;
    logicalHeight = nextLogicalHeight;
    if (wasm) {
      wasm.app_resize(logicalWidth, logicalHeight);
      drawFrame();
    }
  }
}

function viewportTransform() {
  return {
    scaleX: canvas.width / logicalWidth,
    scaleY: canvas.height / logicalHeight,
    offsetX: 0,
    offsetY: 0,
  };
}

function fontFor(token, size) {
  switch (token) {
    case "display":
      return `700 ${size}px ${MONO_STACK}`;
    case "label":
      return `700 ${size}px ${MONO_STACK}`;
    case "body-italic":
      return `italic 400 ${size}px ${MONO_STACK}`;
    default:
      return `400 ${size}px ${MONO_STACK}`;
  }
}

function decodeSceneText(text) {
  const bytes = [];
  let literalStart = 0;
  let index = 0;

  while (index < text.length) {
    if (text[index] === "%" && index + 2 < text.length) {
      const hex = text.slice(index + 1, index + 3);
      if (/^[0-9A-Fa-f]{2}$/.test(hex)) {
        if (literalStart < index) {
          bytes.push(...encoder.encode(text.slice(literalStart, index)));
        }
        bytes.push(Number.parseInt(hex, 16));
        index += 3;
        literalStart = index;
        continue;
      }
    }
    index += 1;
  }

  if (literalStart < text.length) {
    bytes.push(...encoder.encode(text.slice(literalStart)));
  }

  return decoder.decode(Uint8Array.from(bytes));
}

function roundedRectPath(context, x, y, w, h, radius) {
  const r = Math.min(radius, w * 0.5, h * 0.5);
  context.beginPath();
  context.moveTo(x + r, y);
  context.arcTo(x + w, y, x + w, y + h, r);
  context.arcTo(x + w, y + h, x, y + h, r);
  context.arcTo(x, y + h, x, y, r);
  context.arcTo(x, y, x + w, y, r);
  context.closePath();
}

function drawRoundedRect(x, y, w, h, radius) {
  roundedRectPath(ctx, x, y, w, h, radius);
}

function drawRegularPolygon(cx, cy, radius, sides, rotationDeg) {
  const count = Math.max(3, Math.floor(sides));
  const rotation = (rotationDeg * Math.PI) / 180;
  ctx.beginPath();
  for (let i = 0; i < count; i += 1) {
    const angle = rotation - Math.PI / 2 + (i * Math.PI * 2) / count;
    const x = cx + Math.cos(angle) * radius;
    const y = cy + Math.sin(angle) * radius;
    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  }
  ctx.closePath();
}

function drawTriangle(x1, y1, x2, y2, x3, y3) {
  ctx.beginPath();
  ctx.moveTo(x1, y1);
  ctx.lineTo(x2, y2);
  ctx.lineTo(x3, y3);
  ctx.closePath();
}

function getImageAsset(src) {
  if (!imageCache.has(src)) {
    const img = new Image();
    img.decoding = "async";
    img.src = src;
    img.addEventListener("load", () => {
      drawFrame();
    });
    imageCache.set(src, img);
  }
  return imageCache.get(src);
}

function loadImageAsset(src) {
  const img = getImageAsset(src);
  if (img.complete && img.naturalWidth > 0) {
    return Promise.resolve(img);
  }
  return new Promise((resolve, reject) => {
    const handleLoad = () => {
      cleanup();
      resolve(img);
    };
    const handleError = () => {
      cleanup();
      reject(new Error(`Failed to load image asset: ${src}`));
    };
    const cleanup = () => {
      img.removeEventListener("load", handleLoad);
      img.removeEventListener("error", handleError);
    };
    img.addEventListener("load", handleLoad);
    img.addEventListener("error", handleError);
  });
}

function buildTintedImageAsset(src, color) {
  const img = getImageAsset(src);
  if (!(img && img.complete && img.naturalWidth > 0 && img.naturalHeight > 0)) {
    return null;
  }

  const rasterWidth = img.naturalWidth;
  const rasterHeight = img.naturalHeight;
  const tintedCanvas = document.createElement("canvas");
  tintedCanvas.width = rasterWidth;
  tintedCanvas.height = rasterHeight;
  const tintedCtx = tintedCanvas.getContext("2d");
  if (!tintedCtx) {
    return null;
  }

  tintedCtx.clearRect(0, 0, tintedCanvas.width, tintedCanvas.height);
  tintedCtx.imageSmoothingEnabled = true;
  tintedCtx.imageSmoothingQuality = "high";
  tintedCtx.drawImage(img, 0, 0, rasterWidth, rasterHeight);
  tintedCtx.globalCompositeOperation = "source-in";
  tintedCtx.fillStyle = color;
  tintedCtx.fillRect(0, 0, tintedCanvas.width, tintedCanvas.height);
  tintedCtx.globalCompositeOperation = "source-over";
  return tintedCanvas;
}

function getTintedImageAsset(src, color) {
  const key = `${src}|${color.trim().toLowerCase()}`;
  const cached = tintedImageCache.get(key);
  if (cached) {
    return cached;
  }

  const tinted = buildTintedImageAsset(src, color);
  if (tinted) {
    tintedImageCache.set(key, tinted);
  }
  return tinted;
}

async function preloadShellImages() {
  await Promise.allSettled(
    [LOGO_ASSET_PATH, ...COMBAT_ICON_ASSET_PATHS].map((src) => loadImageAsset(src)),
  );
}

function resolveSpriteColor(color) {
  if (spriteColorCache.has(color)) {
    return spriteColorCache.get(color);
  }

  if (!spriteColorProbeCtx) {
    const fallback = [255, 255, 255, 255];
    spriteColorCache.set(color, fallback);
    return fallback;
  }

  spriteColorProbeCtx.clearRect(0, 0, 1, 1);
  spriteColorProbeCtx.fillStyle = color;
  spriteColorProbeCtx.fillRect(0, 0, 1, 1);
  const data = spriteColorProbeCtx.getImageData(0, 0, 1, 1).data;
  const rgba = [data[0], data[1], data[2], data[3]];
  spriteColorCache.set(color, rgba);
  return rgba;
}

function buildEnemySprite(code, color) {
  if (
    !wasm ||
    typeof wasm.enemy_sprite_width !== "function" ||
    typeof wasm.enemy_sprite_height !== "function" ||
    typeof wasm.enemy_sprite_data_ptr !== "function" ||
    typeof wasm.enemy_sprite_data_len !== "function"
  ) {
    return null;
  }

  const width = wasm.enemy_sprite_width(code) >>> 0;
  const height = wasm.enemy_sprite_height(code) >>> 0;
  const len = wasm.enemy_sprite_data_len(code) >>> 0;
  if (!width || !height || !len || len * 8 < width * height) {
    return null;
  }

  const ptr = wasm.enemy_sprite_data_ptr(code);
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len).slice();
  const spriteCanvas = document.createElement("canvas");
  spriteCanvas.width = width;
  spriteCanvas.height = height;
  const spriteCtx = spriteCanvas.getContext("2d");
  if (!spriteCtx) {
    return null;
  }

  const imageData = spriteCtx.createImageData(width, height);
  const pixels = imageData.data;
  const [r, g, b, a] = resolveSpriteColor(color);

  for (let bitIndex = 0; bitIndex < width * height; bitIndex += 1) {
    const byte = bytes[bitIndex >> 3];
    const mask = 0x80 >> (bitIndex & 7);
    if ((byte & mask) === 0) {
      continue;
    }
    const offset = bitIndex * 4;
    pixels[offset] = r;
    pixels[offset + 1] = g;
    pixels[offset + 2] = b;
    pixels[offset + 3] = a;
  }

  spriteCtx.putImageData(imageData, 0, 0);
  return spriteCanvas;
}

function getEnemySprite(code, color) {
  const key = `${code}|${color}`;
  if (!spriteCache.has(key)) {
    spriteCache.set(key, buildEnemySprite(code, color));
  }
  return spriteCache.get(key);
}

function spriteAnimationState(code, color, timeMs) {
  const style = spriteStyleByColor.get(color.toLowerCase()) || { tier: 1, variant: "base" };
  const { tier, variant } = style;
  if (variant === "dim") {
    return { dx: 0, dy: 0, scale: 1, alpha: 1 };
  }

  const t = timeMs * 0.001;
  const seed = code * 0.61803398875;
  const wave = (speed, phase = 0) => Math.sin(t * speed + seed + phase);
  const pulse = (speed, phase = 0) => wave(speed, phase) * 0.5 + 0.5;
  const baseBob = wave(1.1 + (code % 5) * 0.12, 0.4) * 0.006;
  let state;

  switch (variant) {
    case "detailA":
      state = {
        dx: wave(2.8, 0.3) * 0.018,
        dy: baseBob + wave(2.1, 1.2) * 0.012,
        scale: 1 + pulse(3.6, 0.9) * 0.045,
        alpha: 0.76 + pulse(4.4, 0.2) * 0.24,
      };
      break;
    case "detailB":
      state = {
        dx: 0,
        dy: baseBob * 0.6,
        scale: 1 + pulse(1.0, 1.9) * 0.028,
        alpha: 0.82 + pulse(1.8, 0.6) * 0.16,
      };
      break;
    case "detailC":
      state = {
        dx: wave(3.1, 0.8) * 0.014,
        dy: baseBob + wave(4.2, 0.7) * 0.01,
        scale: 1 + pulse(2.6, 1.4) * 0.022,
        alpha: 0.74 + pulse(5.1, 1.2) * 0.2,
      };
      break;
    case "detailD":
      state = {
        dx: wave(2.2, 0.1) * 0.01,
        dy: baseBob + wave(2.0, 2.1) * 0.008,
        scale: 1 + pulse(5.8, 0.4) * 0.06,
        alpha: 0.6 + pulse(6.6, 0.1) * 0.4,
      };
      break;
    case "detailE":
      state = {
        dx: wave(1.7, 0.9) * 0.012,
        dy: baseBob + wave(1.4, 1.7) * 0.012,
        scale: 1 + pulse(2.1, 0.3) * 0.03,
        alpha: 0.84 + pulse(2.4, 0.6) * 0.14,
      };
      break;
    case "base":
    default:
      state = {
        dx: wave(1.5, 0.2) * 0.006,
        dy: baseBob + wave(1.8, 0.5) * 0.012,
        scale: 1 + wave(1.2, 1.1) * 0.018,
        alpha: 0.94 + pulse(1.1, 0.3) * 0.06,
      };
      break;
  }

  if (tier === 2) {
    return {
      dx: state.dx * 1.14,
      dy: state.dy * 1.1,
      scale: 1 + (state.scale - 1) * 1.18,
      alpha: Math.min(1.08, state.alpha * 1.04),
    };
  }

  if (tier === 3) {
    const surge = Math.max(0, wave(6.2, 0.7)) * 0.018;
    const snap = variant === "base" ? 0 : wave(8.0, 0.3) * 0.005;
    return {
      dx: state.dx * 1.32 + snap,
      dy: state.dy * 1.26 - surge * (variant === "detailC" || variant === "detailD" ? 0.45 : 0.28),
      scale: 1 + (state.scale - 1) * 1.45 + surge * 0.4,
      alpha: Math.min(1.14, state.alpha * 1.08 + surge * 0.24),
    };
  }

  return state;
}

function ensureBlurCanvas() {
  if (blurCanvas.width !== canvas.width || blurCanvas.height !== canvas.height) {
    blurCanvas.width = canvas.width;
    blurCanvas.height = canvas.height;
  }
}

function applyBackdropBlur(x, y, w, h, radius, blurAmount, transform) {
  const scale = Math.min(transform.scaleX, transform.scaleY);
  const px = x * transform.scaleX + transform.offsetX;
  const py = y * transform.scaleY + transform.offsetY;
  const pw = w * transform.scaleX;
  const ph = h * transform.scaleY;
  const pr = radius * scale;
  const blurPx = Math.max(1, blurAmount * scale);
  const edgePad = Math.max(2, Math.ceil(blurPx * 3));

  if (
    blurCanvas.width !== canvas.width + edgePad * 2 ||
    blurCanvas.height !== canvas.height + edgePad * 2
  ) {
    blurCanvas.width = canvas.width + edgePad * 2;
    blurCanvas.height = canvas.height + edgePad * 2;
  }

  blurCtx.setTransform(1, 0, 0, 1, 0, 0);
  blurCtx.clearRect(0, 0, blurCanvas.width, blurCanvas.height);
  blurCtx.drawImage(canvas, edgePad, edgePad);

  blurCtx.drawImage(canvas, 0, 0, canvas.width, 1, edgePad, 0, canvas.width, edgePad);
  blurCtx.drawImage(
    canvas,
    0,
    canvas.height - 1,
    canvas.width,
    1,
    edgePad,
    edgePad + canvas.height,
    canvas.width,
    edgePad,
  );
  blurCtx.drawImage(canvas, 0, 0, 1, canvas.height, 0, edgePad, edgePad, canvas.height);
  blurCtx.drawImage(
    canvas,
    canvas.width - 1,
    0,
    1,
    canvas.height,
    edgePad + canvas.width,
    edgePad,
    edgePad,
    canvas.height,
  );
  blurCtx.drawImage(canvas, 0, 0, 1, 1, 0, 0, edgePad, edgePad);
  blurCtx.drawImage(
    canvas,
    canvas.width - 1,
    0,
    1,
    1,
    edgePad + canvas.width,
    0,
    edgePad,
    edgePad,
  );
  blurCtx.drawImage(
    canvas,
    0,
    canvas.height - 1,
    1,
    1,
    0,
    edgePad + canvas.height,
    edgePad,
    edgePad,
  );
  blurCtx.drawImage(
    canvas,
    canvas.width - 1,
    canvas.height - 1,
    1,
    1,
    edgePad + canvas.width,
    edgePad + canvas.height,
    edgePad,
    edgePad,
  );

  ctx.save();
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  drawRoundedRect(px, py, pw, ph, pr);
  ctx.clip();
  ctx.filter = `blur(${blurPx.toFixed(2)}px)`;
  ctx.drawImage(blurCanvas, -edgePad, -edgePad);
  ctx.filter = "none";
  ctx.restore();
}

function renderScene(scene, sceneWidth, sceneHeight) {
  const animationTimeMs = performance.now();
  const transform = {
    scaleX: canvas.width / Math.max(1, sceneWidth),
    scaleY: canvas.height / Math.max(1, sceneHeight),
    offsetX: 0,
    offsetY: 0,
  };

  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "#000000";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  ctx.save();
  ctx.setTransform(transform.scaleX, 0, 0, transform.scaleY, transform.offsetX, transform.offsetY);
  ctx.textBaseline = "alphabetic";

  for (const line of scene.split("\n")) {
    if (!line) {
      continue;
    }
    const parts = line.split("|");
    const opcode = parts[0];

    if (opcode === "CLEAR") {
      ctx.fillStyle = parts[1];
      ctx.fillRect(0, 0, sceneWidth, sceneHeight);
      continue;
    }

    if (opcode === "PUSH") {
      const [, alpha, offsetX, offsetY, scale] = parts;
      const parsedAlpha = Number.parseFloat(alpha);
      const parsedOffsetX = Number.parseFloat(offsetX);
      const parsedOffsetY = Number.parseFloat(offsetY);
      const parsedScale = Number.parseFloat(scale);
      ctx.save();
      ctx.globalAlpha *= Number.isFinite(parsedAlpha) ? parsedAlpha : 1;
      ctx.translate(
        Number.isFinite(parsedOffsetX) ? parsedOffsetX : 0,
        Number.isFinite(parsedOffsetY) ? parsedOffsetY : 0,
      );
      if (Number.isFinite(parsedScale) && parsedScale !== 1) {
        const centerX = sceneWidth * 0.5;
        const centerY = sceneHeight * 0.5;
        ctx.translate(centerX, centerY);
        ctx.scale(parsedScale, parsedScale);
        ctx.translate(-centerX, -centerY);
      }
      continue;
    }

    if (opcode === "POP") {
      ctx.restore();
      continue;
    }

    if (opcode === "RECT") {
      const [, x, y, w, h, radius, fill, stroke, strokeWidth] = parts;
      drawRoundedRect(
        Number.parseFloat(x),
        Number.parseFloat(y),
        Number.parseFloat(w),
        Number.parseFloat(h),
        Number.parseFloat(radius),
      );
      if (fill !== "transparent") {
        ctx.fillStyle = fill;
        ctx.fill();
      }
      if (stroke !== "transparent" && Number.parseFloat(strokeWidth) > 0) {
        ctx.strokeStyle = stroke;
        ctx.lineWidth = Number.parseFloat(strokeWidth);
        ctx.stroke();
      }
      continue;
    }

    if (opcode === "LINE") {
      const [, x1, y1, x2, y2, color, width] = parts;
      ctx.beginPath();
      ctx.moveTo(Number.parseFloat(x1), Number.parseFloat(y1));
      ctx.lineTo(Number.parseFloat(x2), Number.parseFloat(y2));
      ctx.strokeStyle = color;
      ctx.lineWidth = Number.parseFloat(width);
      ctx.stroke();
      continue;
    }

    if (opcode === "BLUR") {
      const [, x, y, w, h, radius, amount] = parts;
      applyBackdropBlur(
        Number.parseFloat(x),
        Number.parseFloat(y),
        Number.parseFloat(w),
        Number.parseFloat(h),
        Number.parseFloat(radius),
        Number.parseFloat(amount),
        transform,
      );
      continue;
    }

    if (opcode === "IMAGE") {
      const [, x, y, w, h, src, alpha] = parts;
      const img = getImageAsset(src);
      if (img && img.complete) {
        ctx.save();
        ctx.globalAlpha *= Number.parseFloat(alpha);
        ctx.drawImage(
          img,
          Number.parseFloat(x),
          Number.parseFloat(y),
          Number.parseFloat(w),
          Number.parseFloat(h),
        );
        ctx.restore();
      }
      continue;
    }

    if (opcode === "TIMAGE") {
      const [, x, y, w, h, src, color, alpha] = parts;
      const img = getTintedImageAsset(src, color);
      if (img) {
        ctx.save();
        ctx.globalAlpha *= Number.parseFloat(alpha);
        ctx.imageSmoothingEnabled = true;
        ctx.imageSmoothingQuality = "high";
        ctx.drawImage(
          img,
          Number.parseFloat(x),
          Number.parseFloat(y),
          Number.parseFloat(w),
          Number.parseFloat(h),
        );
        ctx.restore();
      }
      continue;
    }

    if (opcode === "SPRITE") {
      const [, x, y, w, h, code, color, alpha] = parts;
      const sprite = getEnemySprite(Number.parseInt(code, 10), color);
      if (sprite) {
        const xValue = Number.parseFloat(x);
        const yValue = Number.parseFloat(y);
        const wValue = Number.parseFloat(w);
        const hValue = Number.parseFloat(h);
        const animation = spriteAnimationState(
          Number.parseInt(code, 10),
          color,
          animationTimeMs,
        );
        const parsedAlpha = Number.parseFloat(alpha);
        const spriteAlpha = Number.isFinite(parsedAlpha) ? parsedAlpha : 1;
        const clampedAlpha = Math.min(1, Math.max(0, spriteAlpha * animation.alpha));
        const centerX = xValue + wValue * 0.5 + animation.dx * wValue;
        const centerY = yValue + hValue * 0.5 + animation.dy * hValue;
        ctx.save();
        ctx.globalAlpha = ctx.globalAlpha * clampedAlpha;
        ctx.imageSmoothingEnabled = false;
        ctx.translate(centerX, centerY);
        ctx.scale(animation.scale, animation.scale);
        ctx.drawImage(sprite, -wValue * 0.5, -hValue * 0.5, wValue, hValue);
        ctx.restore();
      }
      continue;
    }

    if (opcode === "POLY") {
      const [, cx, cy, radius, sides, rotationDeg, fill, stroke, strokeWidth] = parts;
      drawRegularPolygon(
        Number.parseFloat(cx),
        Number.parseFloat(cy),
        Number.parseFloat(radius),
        Number.parseInt(sides, 10),
        Number.parseFloat(rotationDeg),
      );
      if (fill !== "transparent") {
        ctx.fillStyle = fill;
        ctx.fill();
      }
      if (stroke !== "transparent" && Number.parseFloat(strokeWidth) > 0) {
        ctx.strokeStyle = stroke;
        ctx.lineWidth = Number.parseFloat(strokeWidth);
        ctx.stroke();
      }
      continue;
    }

    if (opcode === "TRI") {
      const [, x1, y1, x2, y2, x3, y3, fill, stroke, strokeWidth] = parts;
      drawTriangle(
        Number.parseFloat(x1),
        Number.parseFloat(y1),
        Number.parseFloat(x2),
        Number.parseFloat(y2),
        Number.parseFloat(x3),
        Number.parseFloat(y3),
      );
      if (fill !== "transparent") {
        ctx.fillStyle = fill;
        ctx.fill();
      }
      if (stroke !== "transparent" && Number.parseFloat(strokeWidth) > 0) {
        ctx.strokeStyle = stroke;
        ctx.lineWidth = Number.parseFloat(strokeWidth);
        ctx.stroke();
      }
      continue;
    }

    if (opcode === "TEXT") {
      const [, x, y, size, align, color, fontToken, text] = parts;
      ctx.textAlign = align;
      ctx.fillStyle = color;
      ctx.font = fontFor(fontToken, Number.parseFloat(size));
      ctx.fillText(decodeSceneText(text), Number.parseFloat(x), Number.parseFloat(y));
    }
  }

  ctx.restore();
}

function drawFrame() {
  const remoteFrame = multiplayer.currentRemoteFrame();
  if (!wasm && !remoteFrame) {
    hidePlayerNameInput();
    return;
  }

  if (remoteFrame && !multiplayer.guestRendersLocalState()) {
    hidePlayerNameInput();
    renderScene(remoteFrame.scene, remoteFrame.width, remoteFrame.height);
    const blocking = multiplayer.currentBlockingScreen();
    if (blocking) {
      renderMultiplayerBlockingScreen(blocking);
    }
    return;
  }

  syncBootScreenState();
  const ptr = wasm.frame_ptr();
  const len = wasm.frame_len();
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
  const scene = decoder.decode(bytes);
  renderScene(scene, logicalWidth, logicalHeight);
  syncRunSaveSnapshot();
  syncPartySnapshot();
  const blocking = multiplayer.currentBlockingScreen();
  if (blocking || multiplayer.isRoomOpen()) {
    hidePlayerNameInput();
  } else {
    syncPlayerNameInput();
  }
  if (blocking) {
    renderMultiplayerBlockingScreen(blocking);
  }
  multiplayer.afterLocalRender(scene, {
    width: logicalWidth,
    height: logicalHeight,
  });
}

function wrapCanvasText(context, text, maxWidth) {
  const words = String(text || "").trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) {
    return [];
  }
  const lines = [];
  let line = "";
  for (const word of words) {
    const candidate = line ? `${line} ${word}` : word;
    if (line && context.measureText(candidate).width > maxWidth) {
      lines.push(line);
      line = word;
    } else {
      line = candidate;
    }
  }
  if (line) {
    lines.push(line);
  }
  return lines;
}

function lerpNumber(start, end, t) {
  return start + (end - start) * t;
}

function parseRgbaColor(color) {
  const rgbaMatch = String(color || "").match(
    /^rgba\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*([0-9.]+)\s*\)$/i,
  );
  if (rgbaMatch) {
    return {
      r: Number.parseInt(rgbaMatch[1], 10),
      g: Number.parseInt(rgbaMatch[2], 10),
      b: Number.parseInt(rgbaMatch[3], 10),
      a: Number.parseFloat(rgbaMatch[4]),
    };
  }
  const hexMatch = String(color || "").match(/^#([0-9a-f]{6})$/i);
  if (hexMatch) {
    return {
      r: Number.parseInt(hexMatch[1].slice(0, 2), 16),
      g: Number.parseInt(hexMatch[1].slice(2, 4), 16),
      b: Number.parseInt(hexMatch[1].slice(4, 6), 16),
      a: 1,
    };
  }
  return { r: 51, g: 255, b: 102, a: 0.85 };
}

function rgbaColor({ r, g, b }, alpha) {
  return `rgba(${r}, ${g}, ${b}, ${Math.max(0, Math.min(1, alpha)).toFixed(3)})`;
}

function drawGameUiTile(rect, { radius = GAME_BUTTON_RADIUS, stroke = GAME_UI_STROKE_PANEL } = {}) {
  const color = parseRgbaColor(stroke);
  const strokeColor = rgbaColor(color, color.a + GAME_UI_STROKE_ALPHA_BOOST);
  const fillColor = rgbaColor(color, GAME_UI_FILL_ALPHA);
  ctx.save();
  ctx.fillStyle = fillColor;
  ctx.strokeStyle = strokeColor;
  ctx.lineWidth = GAME_UI_STROKE_WIDTH;
  roundedRectPath(ctx, rect.x, rect.y, rect.width, rect.height, radius);
  ctx.fill();
  ctx.stroke();
  ctx.restore();
}

function primaryButtonPulse() {
  return 0.55 + 0.45 * Math.abs(Math.sin(performance.now() * 0.0025));
}

function gameButtonTextBaseline(rect, fontSize) {
  return rect.y + rect.height * 0.5 + fontSize * 0.32;
}

function fitGameButtonMetrics(label, maxWidth) {
  const text = String(label || "");
  const availableWidth = Math.max(1, maxWidth);
  for (let step = 0; step <= 24; step += 1) {
    const t = step / 24;
    const fontSize = lerpNumber(GAME_BUTTON_FONT_SIZE, GAME_BUTTON_MIN_FONT_SIZE, t);
    const padX = lerpNumber(GAME_BUTTON_PAD_X, GAME_BUTTON_MIN_PAD_X, t);
    const padY = lerpNumber(GAME_BUTTON_PAD_Y, GAME_BUTTON_MIN_PAD_Y, t);
    ctx.save();
    ctx.font = fontFor("label", fontSize);
    const width = ctx.measureText(text).width + padX * 2;
    ctx.restore();
    const height = fontSize + padY * 2;
    if (width <= availableWidth || step === 24) {
      return {
        width: Math.min(width, availableWidth),
        height,
        fontSize,
        padX,
        padY,
      };
    }
  }
  return {
    width: availableWidth,
    height: GAME_BUTTON_MIN_FONT_SIZE + GAME_BUTTON_MIN_PAD_Y * 2,
    fontSize: GAME_BUTTON_MIN_FONT_SIZE,
    padX: GAME_BUTTON_MIN_PAD_X,
    padY: GAME_BUTTON_MIN_PAD_Y,
  };
}

function drawGamePrimaryButton(label, rect, { hovered = false } = {}) {
  drawGameUiTile(rect, { radius: GAME_BUTTON_RADIUS, stroke: GAME_UI_STROKE_START });
  ctx.save();
  ctx.textAlign = "center";
  ctx.fillStyle = rgbaColor(parseRgbaColor(TERM_GREEN), hovered ? 1 : primaryButtonPulse());
  ctx.font = fontFor("label", rect.fontSize || GAME_BUTTON_FONT_SIZE);
  ctx.fillText(
    String(label || ""),
    rect.x + rect.width * 0.5,
    gameButtonTextBaseline(rect, rect.fontSize || GAME_BUTTON_FONT_SIZE),
  );
  ctx.restore();
}

function blockingScreenBannerLayout(screen) {
  if (screen?.presentation !== "banner") {
    return null;
  }
  const title = screen.title || "Waiting";
  const subtitle = screen.subtitle || "";
  const titleFontSize = 24;
  const subtitleFontSize = 16;
  const horizontalPad = 24;
  const topPad = 16;
  const subtitleGap = subtitle ? 10 : 0;
  const bottomPad = subtitle ? 16 : 14;
  const maxWidth = Math.max(220, logicalWidth - 48);
  ctx.font = fontFor("display", titleFontSize);
  const titleWidth = ctx.measureText(title).width;
  ctx.font = fontFor("body", subtitleFontSize);
  const subtitleLines = wrapCanvasText(ctx, subtitle, maxWidth - horizontalPad * 2);
  const subtitleWidth = subtitleLines
    .map((line) => ctx.measureText(line).width)
    .reduce((max, width) => Math.max(max, width), 0);
  const width = Math.min(
    Math.max(titleWidth, subtitleWidth) + horizontalPad * 2,
    maxWidth,
  );
  const subtitleLineHeight = subtitleFontSize + 5;
  const height =
    topPad +
    titleFontSize +
    (subtitleLines.length ? subtitleGap + subtitleLines.length * subtitleLineHeight : 0) +
    bottomPad;
  return {
    title,
    subtitleLines,
    titleFontSize,
    subtitleFontSize,
    subtitleGap,
    subtitleLineHeight,
    topPad,
    x: logicalWidth * 0.5 - width * 0.5,
    y: Math.max(0, logicalHeight * 0.5 - height * 0.5),
    width,
    height,
  };
}

function blockingScreenBannerRect(screen) {
  const layout = blockingScreenBannerLayout(screen);
  return layout
    ? {
        x: layout.x,
        y: layout.y,
        width: layout.width,
        height: layout.height,
      }
    : null;
}

function renderMultiplayerBlockingScreen(screen) {
  if (!screen) {
    return;
  }
  const transform = viewportTransform();
  const presentation = screen.presentation === "banner" ? "banner" : "fullscreen";
  ctx.save();
  ctx.setTransform(
    transform.scaleX,
    0,
    0,
    transform.scaleY,
    transform.offsetX,
    transform.offsetY,
  );
  if (presentation === "banner") {
    const layout = blockingScreenBannerLayout(screen);
    if (!layout) {
      ctx.restore();
      return;
    }
    drawGameUiTile(layout, { stroke: GAME_UI_STROKE_PANEL });
    ctx.textAlign = "center";
    ctx.fillStyle = TERM_GREEN_SOFT;
    ctx.font = fontFor("display", layout.titleFontSize);
    ctx.fillText(
      layout.title,
      logicalWidth * 0.5,
      layout.y + layout.topPad + layout.titleFontSize,
    );
    if (layout.subtitleLines.length) {
      ctx.fillStyle = TERM_INK;
      ctx.font = fontFor("body", layout.subtitleFontSize);
      layout.subtitleLines.forEach((line, index) => {
        ctx.fillText(
          line,
          logicalWidth * 0.5,
          layout.y +
            layout.topPad +
            layout.titleFontSize +
            layout.subtitleGap +
            layout.subtitleFontSize +
            index * layout.subtitleLineHeight,
        );
      });
    }
    ctx.restore();
    return;
  }
  ctx.fillStyle = "#000000";
  ctx.fillRect(0, 0, logicalWidth, logicalHeight);
  ctx.textAlign = "center";
  const maxTextWidth = Math.max(220, logicalWidth - 56);
  const titleFontSize = Math.min(34, Math.max(26, logicalWidth * 0.075));
  const subtitleFontSize = Math.min(18, Math.max(15, logicalWidth * 0.044));
  const titleY = logicalHeight * 0.4;
  ctx.fillStyle = TERM_GREEN_SOFT;
  ctx.font = fontFor("display", titleFontSize);
  ctx.fillText(screen.title || "Waiting", logicalWidth * 0.5, titleY);
  ctx.fillStyle = TERM_INK;
  ctx.font = fontFor("body", subtitleFontSize);
  const subtitleLines = wrapCanvasText(ctx, screen.subtitle || "", maxTextWidth);
  const subtitleLineHeight = subtitleFontSize + 6;
  subtitleLines.forEach((line, index) => {
    ctx.fillText(line, logicalWidth * 0.5, logicalHeight * 0.49 + index * subtitleLineHeight);
  });
  const actionRect = blockingScreenActionRect(screen);
  if (actionRect) {
    drawGamePrimaryButton(screen.actionLabel, actionRect);
  }
  ctx.restore();
}

function blockingScreenActionRect(screen) {
  if (!screen?.actionLabel) {
    return null;
  }
  const horizontalMargin = logicalWidth >= 240 ? 48 : 16;
  const maxWidth = Math.max(1, logicalWidth - horizontalMargin * 2);
  const metrics = fitGameButtonMetrics(screen.actionLabel, maxWidth);
  return {
    x: logicalWidth * 0.5 - metrics.width * 0.5,
    y: logicalHeight * 0.62,
    width: metrics.width,
    height: metrics.height,
    fontSize: metrics.fontSize,
  };
}

function pointInRect(point, rect) {
  return (
    !!point &&
    !!rect &&
    point.x >= rect.x &&
    point.x <= rect.x + rect.width &&
    point.y >= rect.y &&
    point.y <= rect.y + rect.height
  );
}

function toCanvasPoint(event) {
  const rect = canvas.getBoundingClientRect();
  return {
    x: (event.clientX - rect.left) * (logicalWidth / rect.width),
    y: (event.clientY - rect.top) * (logicalHeight / rect.height),
  };
}

function normalizedCanvasPoint(event) {
  const rect = canvas.getBoundingClientRect();
  return {
    xNorm: rect.width > 0 ? (event.clientX - rect.left) / rect.width : 0,
    yNorm: rect.height > 0 ? (event.clientY - rect.top) / rect.height : 0,
  };
}

function clearHover() {
  if (!wasm) {
    return;
  }
  wasm.pointer_move(-1, -1);
}

function mixEntropy() {
  if (!wasm || typeof wasm.app_mix_entropy !== "function") {
    return;
  }

  let low = 0;
  let high = 0;
  if (window.crypto?.getRandomValues) {
    const values = new Uint32Array(2);
    window.crypto.getRandomValues(values);
    [low, high] = values;
  } else {
    const now = Date.now() >>> 0;
    low = ((Math.random() * 0x1_0000_0000) ^ now) >>> 0;
    high = ((performance.now() * 1000) ^ (Math.random() * 0x1_0000_0000)) >>> 0;
  }
  wasm.app_mix_entropy(low, high);
}

function readStoredRun() {
  try {
    return window.localStorage.getItem(ACTIVE_RUN_STORAGE_KEY);
  } catch {
    return null;
  }
}

function writeStoredRun(raw) {
  try {
    window.localStorage.setItem(ACTIVE_RUN_STORAGE_KEY, raw);
    return true;
  } catch (error) {
    console.error(error);
    return false;
  }
}

function clearStoredRun() {
  try {
    window.localStorage.removeItem(ACTIVE_RUN_STORAGE_KEY);
  } catch {}
  lastRunSaveRaw = null;
  lastPartySnapshot = null;
  lastRunSummary = null;
  multiplayer.setLocalPartySnapshot(null);
  multiplayer.setLocalSummary(null);
}

function readStoredLanguage() {
  try {
    const raw = window.localStorage.getItem(LANGUAGE_STORAGE_KEY);
    if (raw === "1") {
      return 1;
    }
  } catch {}
  return 0;
}

function writeStoredLanguage(code) {
  try {
    window.localStorage.setItem(LANGUAGE_STORAGE_KEY, code === 1 ? "1" : "0");
    return true;
  } catch (error) {
    console.error(error);
    return false;
  }
}

function readStoredBackgroundMode() {
  try {
    const raw = window.localStorage.getItem(BACKGROUND_MODE_STORAGE_KEY);
    if (raw === "1") {
      return 1;
    }
  } catch {}
  return 0;
}

function writeStoredBackgroundMode(code) {
  try {
    window.localStorage.setItem(BACKGROUND_MODE_STORAGE_KEY, code === 1 ? "1" : "0");
    return true;
  } catch (error) {
    console.error(error);
    return false;
  }
}

function readStoredPlayerName() {
  try {
    return window.localStorage.getItem(PLAYER_NAME_STORAGE_KEY) ?? "";
  } catch {
    return "";
  }
}

function writeStoredPlayerName(raw) {
  try {
    if (typeof raw === "string" && raw.length > 0) {
      window.localStorage.setItem(PLAYER_NAME_STORAGE_KEY, raw);
    } else {
      window.localStorage.removeItem(PLAYER_NAME_STORAGE_KEY);
    }
    return true;
  } catch (error) {
    console.error(error);
    return false;
  }
}

function readAppPlayerName() {
  if (
    !wasm ||
    typeof wasm.app_player_name_len !== "function" ||
    typeof wasm.app_player_name_ptr !== "function"
  ) {
    return "";
  }

  const len = wasm.app_player_name_len();
  if (!len) {
    return "";
  }
  const ptr = wasm.app_player_name_ptr();
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
  return decoder.decode(bytes.slice());
}

function setAppPlayerName(raw) {
  if (
    !wasm ||
    typeof wasm.prepare_player_name_buffer !== "function" ||
    typeof wasm.app_set_player_name_from_buffer !== "function"
  ) {
    return false;
  }

  const normalized = limitPlayerNameInputValue(raw);
  const bytes = encoder.encode(normalized);
  const ptr = wasm.prepare_player_name_buffer(bytes.length);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
  return !!wasm.app_set_player_name_from_buffer(bytes.length);
}

function isAllowedPlayerNameCharacter(char) {
  return (
    /^[\p{L}\p{N}]$/u.test(char) ||
    char === " " ||
    char === "-" ||
    char === "_" ||
    char === "." ||
    char === "'"
  );
}

function limitPlayerNameInputValue(raw) {
  return Array.from(String(raw))
    .filter((char) => isAllowedPlayerNameCharacter(char))
    .slice(0, PLAYER_NAME_MAX_CHARS)
    .join("");
}

function setAppPlayerNameInputFocused(focused) {
  if (!wasm || typeof wasm.app_set_player_name_input_focused !== "function") {
    return false;
  }

  wasm.app_set_player_name_input_focused(focused ? 1 : 0);
  return true;
}

function setPlayerNameEditingActive(active) {
  playerNameEditingActive = !!active;
  if (playerNameEditingActive && document.activeElement !== playerNameInput) {
    playerNameInput.value = readAppPlayerName();
  }
  setAppPlayerNameInputFocused(playerNameEditingActive);
  if (!playerNameEditingActive && document.activeElement === playerNameInput) {
    playerNameInput.blur();
  }
  if (
    playerNameEditingActive &&
    document.activeElement !== playerNameInput &&
    lastMouseClientPoint
  ) {
    showTypingPointerOverlay();
  } else {
    hideTypingPointerOverlay();
  }
  syncPlayerNameOverlay();
}

function updatePlayerNameFromKeyboard(nextValue) {
  const limitedValue = limitPlayerNameInputValue(nextValue);
  if (limitedValue === playerNameInput.value) {
    return false;
  }
  playerNameInput.value = limitedValue;
  setAppPlayerName(limitedValue);
  syncStoredPlayerName();
  drawFrame();
  return true;
}

function handlePlayerNameEditingKey(event) {
  if (!playerNameEditingActive || !wasm || document.activeElement === playerNameInput) {
    return false;
  }

  if (event.key === "Escape") {
    event.preventDefault();
    setPlayerNameEditingActive(false);
    drawFrame();
    return true;
  }

  if (event.key === "Enter") {
    event.preventDefault();
    setPlayerNameEditingActive(false);
    drawFrame();
    return true;
  }

  const currentValue = playerNameInput.value;

  if (event.key === "Backspace") {
    event.preventDefault();
    if (updatePlayerNameFromKeyboard(Array.from(currentValue).slice(0, -1).join(""))) {
      showTypingPointerOverlay();
    }
    return true;
  }

  if (event.key === "Delete") {
    event.preventDefault();
    if (updatePlayerNameFromKeyboard("")) {
      showTypingPointerOverlay();
    }
    return true;
  }

  if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
    event.preventDefault();
    if (updatePlayerNameFromKeyboard(currentValue + event.key)) {
      showTypingPointerOverlay();
    }
    return true;
  }

  return false;
}

function parsePartySnapshot(raw) {
  if (typeof raw !== "string" || raw.length === 0) {
    return null;
  }

  try {
    return JSON.parse(raw);
  } catch (error) {
    console.error(error);
  }

  return null;
}

function summarizePartySnapshot(snapshot) {
  if (!snapshot || typeof snapshot !== "object") {
    return null;
  }
  const slots = Array.isArray(snapshot.slots) ? snapshot.slots : null;
  if (!slots?.length) {
    return null;
  }
  const localSlotIndex = Number.isInteger(snapshot.local_slot) ? snapshot.local_slot : 0;
  const localSlot =
    slots.find((slot) => Number.isInteger(slot?.slot) && slot.slot === localSlotIndex) ||
    slots[localSlotIndex];
  if (!localSlot || !localSlot.in_combat) {
    return null;
  }
  return {
    inCombat: true,
    hp: Number.isFinite(localSlot.hp) ? localSlot.hp : 0,
    maxHp: Number.isFinite(localSlot.max_hp) ? localSlot.max_hp : 0,
    block: Number.isFinite(localSlot.block) ? localSlot.block : 0,
  };
}

function syncRemotePartySlotToHost(slot, update) {
  if (
    !wasm ||
    typeof wasm.prepare_party_slot_name_buffer !== "function" ||
    typeof wasm.app_sync_remote_party_slot_from_buffer !== "function"
  ) {
    return false;
  }

  const name = typeof update?.name === "string" ? update.name : "";
  const bytes = encoder.encode(name);
  const ptr = wasm.prepare_party_slot_name_buffer(bytes.length);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
  const updated = wasm.app_sync_remote_party_slot_from_buffer(
    slot,
    update?.connected ? 1 : 0,
    update?.ready ? 1 : 0,
    update?.inRun ? 1 : 0,
    update?.inCombat ? 1 : 0,
    update?.alive !== false ? 1 : 0,
    Number.isFinite(update?.hp) ? Math.round(update.hp) : 0,
    Number.isFinite(update?.maxHp) ? Math.round(update.maxHp) : 0,
    Number.isFinite(update?.block) ? Math.round(update.block) : 0,
    bytes.length,
  );
  syncPartySnapshot();
  syncRunSaveSnapshot();
  return !!updated;
}

function disconnectRemotePartySlotOnHost(slot) {
  if (!wasm || typeof wasm.app_disconnect_remote_party_slot !== "function") {
    return false;
  }
  const updated = wasm.app_disconnect_remote_party_slot(slot);
  syncPartySnapshot();
  syncRunSaveSnapshot();
  return !!updated;
}

function clearRemotePartySlotOnHost(slot) {
  if (!wasm || typeof wasm.app_clear_remote_party_slot !== "function") {
    return false;
  }
  const updated = wasm.app_clear_remote_party_slot(slot);
  syncPartySnapshot();
  syncRunSaveSnapshot();
  return !!updated;
}

function syncPartySnapshot() {
  if (
    !wasm ||
    typeof wasm.party_snapshot_generation !== "function" ||
    typeof wasm.party_snapshot_len !== "function" ||
    typeof wasm.party_snapshot_ptr !== "function"
  ) {
    return false;
  }

  const generation = wasm.party_snapshot_generation();
  if (generation === lastPartySnapshotGeneration) {
    return false;
  }
  lastPartySnapshotGeneration = generation;

  const len = wasm.party_snapshot_len();
  if (!len) {
    lastPartySnapshot = null;
    lastRunSummary = null;
    multiplayer.setLocalPartySnapshot(null);
    multiplayer.setLocalSummary(null);
    return true;
  }

  const ptr = wasm.party_snapshot_ptr();
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
  const raw = decoder.decode(bytes.slice());
  lastPartySnapshot = parsePartySnapshot(raw);
  lastRunSummary = summarizePartySnapshot(lastPartySnapshot);
  multiplayer.setLocalSummary(lastRunSummary);
  multiplayer.setLocalPartySnapshot(lastPartySnapshot);

  try {
    const payload = lastPartySnapshot;
    if (Number.isFinite(payload?.configured_party_size)) {
      multiplayer.setConfiguredPartySize(payload.configured_party_size);
    }
  } catch (error) {
    console.error(error);
  }

  return true;
}

const multiplayer = createMultiplayerController({
  getLocalName: () => readAppPlayerName() || readStoredPlayerName() || "Player",
  getLocalRunSnapshot: () => lastRunSaveRaw || readStoredRun(),
  getLocalRunSnapshotVersion: () => lastRunSaveGeneration,
  applyGuestRunSnapshot: (raw, slot) => restoreGuestRunSnapshot(raw, slot),
  encodePairPayload: (payload) => encodePairPayloadRust(payload),
  buildPairTransportFrames: (raw, chunkChars) =>
    buildPairTransportFramesRust(raw, chunkChars),
  resetPairTransportAssembly: () => resetPairTransportAssemblyRust(),
  consumePairTransportText: (raw) => consumePairTransportTextRust(raw),
  requestScannerStream: () => e2eSupport?.createFakeScannerStream?.() ?? null,
  rejectGuestCombatAction: (raw, slot) => {
    if (
      wasm &&
      typeof wasm.app_clear_local_multiplayer_pending_combat_action === "function"
    ) {
      wasm.app_clear_local_multiplayer_pending_combat_action();
    }
    return restoreGuestRunSnapshot(raw, slot);
  },
  startHostRun: () => {
    if (!wasm || typeof wasm.app_start_multiplayer_run !== "function") {
      return false;
    }
    const started = !!wasm.app_start_multiplayer_run();
    syncRunSaveSnapshot();
    syncPartySnapshot();
    return started;
  },
  resumeHostRun: () => {
    if (!wasm || typeof wasm.app_resume_multiplayer_run !== "function") {
      return false;
    }
    const resumed = !!wasm.app_resume_multiplayer_run();
    syncRunSaveSnapshot();
    syncPartySnapshot();
    return resumed;
  },
  hasSavedRun: () => !!(lastRunSaveRaw || readStoredRun()),
  restoreSavedRun: () => {
    const raw = lastRunSaveRaw || readStoredRun();
    if (typeof raw !== "string" || !raw.length) {
      return false;
    }
    const restored = restoreRunSnapshotRaw(raw);
    if (restored) {
      drawFrame();
    }
    return restored;
  },
  returnToMenu: () => {
    if (!wasm || typeof wasm.app_return_to_menu !== "function") {
      return false;
    }
    const returned = !!wasm.app_return_to_menu();
    syncRunSaveSnapshot();
    syncPartySnapshot();
    drawFrame();
    return returned;
  },
  getConfiguredPartySize: () =>
    !!(wasm && typeof wasm.app_party_size === "function") ? wasm.app_party_size() : 2,
  setConfiguredPartySize: (size) => {
    if (wasm && typeof wasm.app_set_party_size === "function") {
      wasm.app_set_party_size(size);
    }
  },
  syncHostPartySlot: (slot, update) => syncRemotePartySlotToHost(slot, update),
  disconnectHostPartySlot: (slot) => disconnectRemotePartySlotOnHost(slot),
  clearHostPartySlot: (slot) => clearRemotePartySlotOnHost(slot),
  isBootScreen: () =>
    !!(wasm && typeof wasm.app_is_boot_screen === "function" && wasm.app_is_boot_screen()),
  requestRender: () => drawFrame(),
  mapGuestPointerInput: (kind, normalizedPoint) => {
    if (!wasm || !lastPartySnapshot) {
      return null;
    }
    const x =
      Number.isFinite(normalizedPoint?.xNorm) && Number.isFinite(logicalWidth)
        ? normalizedPoint.xNorm * logicalWidth
        : 0;
    const y =
      Number.isFinite(normalizedPoint?.yNorm) && Number.isFinite(logicalHeight)
        ? normalizedPoint.yNorm * logicalHeight
        : 0;
    if (kind !== "pointer_down") {
      return null;
    }
    if (
      (lastPartySnapshot.screen === "map" || lastPartySnapshot.screen === "combat") &&
      typeof wasm.app_menu_button_hit_at_point === "function" &&
      wasm.app_menu_button_hit_at_point(x, y)
    ) {
      return { kind: "local_pointer" };
    }
    if (
      (lastPartySnapshot.screen === "opening_intro" || lastPartySnapshot.screen === "level_intro") &&
      typeof wasm.app_continue_button_hit_at_point === "function" &&
      wasm.app_continue_button_hit_at_point(x, y)
    ) {
      return {
        kind:
          lastPartySnapshot.screen === "opening_intro"
            ? "opening_intro_action"
            : "level_intro_action",
      };
    }
    if (
      lastPartySnapshot.screen === "module_select" &&
      typeof wasm.app_module_select_card_index_at_point === "function"
    ) {
      const index = wasm.app_module_select_card_index_at_point(x, y);
      if (Number.isInteger(index) && index >= 0) {
        return {
          kind: "module_select",
          index,
        };
      }
    }
    if (lastPartySnapshot.screen === "reward") {
      if (
        typeof wasm.app_reward_skip_hit_at_point === "function" &&
        wasm.app_reward_skip_hit_at_point(x, y)
      ) {
        return {
          kind: "reward_skip",
        };
      }
      if (typeof wasm.app_reward_card_index_at_point === "function") {
        const index = wasm.app_reward_card_index_at_point(x, y);
        if (Number.isInteger(index) && index >= 0) {
          return {
            kind: "reward_select",
            index,
          };
        }
      }
    }
    if (
      lastPartySnapshot.screen === "event" &&
      typeof wasm.app_event_choice_index_at_point === "function"
    ) {
      const index = wasm.app_event_choice_index_at_point(x, y);
      if (Number.isInteger(index) && index >= 0) {
        return {
          kind: "event_choice",
          index,
        };
      }
    }
    if (
      lastPartySnapshot.screen === "combat" &&
      typeof wasm.app_handle_local_multiplayer_combat_pointer_down === "function"
    ) {
      const actionCode = wasm.app_handle_local_multiplayer_combat_pointer_down(x, y);
      drawFrame();
      if (Number.isInteger(actionCode) && actionCode > 0) {
        return {
          kind: "combat_action",
          actionCode,
        };
      }
    }
    return null;
  },
  mapGuestKeyInput: (keyCode) => {
    if (!wasm || !lastPartySnapshot) {
      return null;
    }
    if (lastPartySnapshot.screen === "reward") {
      if (keyCode === 48) {
        return { kind: "reward_skip" };
      }
      if (keyCode >= 49 && keyCode <= 57) {
        const index = keyCode - 49;
        return { kind: "reward_select", index };
      }
      return null;
    }
    if (lastPartySnapshot.screen === "event") {
      if (keyCode >= 49 && keyCode <= 57) {
        const index = keyCode - 49;
        return { kind: "event_choice", index };
      }
      return null;
    }
    if (
      lastPartySnapshot.screen !== "combat" ||
      typeof wasm.app_handle_local_multiplayer_combat_key !== "function"
    ) {
      return null;
    }
    const actionCode = wasm.app_handle_local_multiplayer_combat_key(keyCode);
    drawFrame();
    if (Number.isInteger(actionCode) && actionCode > 0) {
      return {
        kind: "combat_action",
        actionCode,
      };
    }
    return null;
  },
  applyHostInput: (payload) => {
    if (!wasm) {
      return false;
    }

    mixEntropy();
    const slot = Number.isInteger(payload?.slot) ? payload.slot : null;
    if (
      payload.kind === "combat_action" &&
      slot !== null &&
      Number.isInteger(payload.actionCode) &&
      typeof wasm.app_apply_multiplayer_combat_action_code === "function"
    ) {
      const applied =
        wasm.app_apply_multiplayer_combat_action_code(slot, payload.actionCode >>> 0) !== 0;
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return applied;
    }
    const beginScopedInput =
      slot !== null &&
      typeof wasm.app_begin_remote_input_slot === "function" &&
      typeof wasm.app_end_remote_input_slot === "function";
    if (beginScopedInput) {
      wasm.app_begin_remote_input_slot(slot);
    }
    const x =
      Number.isFinite(payload.xNorm) && Number.isFinite(logicalWidth)
        ? payload.xNorm * logicalWidth
        : 0;
    const y =
      Number.isFinite(payload.yNorm) && Number.isFinite(logicalHeight)
        ? payload.yNorm * logicalHeight
        : 0;

    if (payload.kind === "pointer_move") {
      wasm.pointer_move(x, y);
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      return true;
    }
    if (payload.kind === "pointer_down") {
      wasm.pointer_down(x, y);
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "pointer_up") {
      wasm.pointer_up(x, y);
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "opening_intro_action") {
      if (typeof wasm.app_finish_opening_intro === "function") {
        wasm.app_finish_opening_intro();
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "level_intro_action") {
      if (typeof wasm.app_continue_level_intro === "function") {
        wasm.app_continue_level_intro();
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "module_select" && Number.isFinite(payload.index)) {
      if (typeof wasm.app_claim_module_select === "function") {
        wasm.app_claim_module_select(Math.max(0, Math.round(payload.index)));
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "reward_select" && Number.isFinite(payload.index)) {
      if (typeof wasm.app_claim_reward === "function") {
        wasm.app_claim_reward(Math.max(0, Math.round(payload.index)));
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "reward_skip") {
      if (typeof wasm.app_skip_reward === "function") {
        wasm.app_skip_reward();
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "event_choice" && Number.isFinite(payload.index)) {
      if (typeof wasm.app_claim_event_choice === "function") {
        wasm.app_claim_event_choice(Math.max(0, Math.round(payload.index)));
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return;
    }
    if (payload.kind === "rest_heal") {
      if (typeof wasm.app_claim_rest_heal === "function") {
        wasm.app_claim_rest_heal();
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "rest_upgrade" && Number.isFinite(payload.index)) {
      if (typeof wasm.app_claim_rest_upgrade === "function") {
        wasm.app_claim_rest_upgrade(Math.max(0, Math.round(payload.index)));
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "shop_buy" && Number.isFinite(payload.index)) {
      if (typeof wasm.app_claim_shop_offer === "function") {
        wasm.app_claim_shop_offer(Math.max(0, Math.round(payload.index)));
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "shop_leave") {
      if (typeof wasm.app_leave_shop === "function") {
        wasm.app_leave_shop();
      }
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (payload.kind === "key_down" && Number.isFinite(payload.keyCode)) {
      wasm.key_down(payload.keyCode);
      if (beginScopedInput) {
        wasm.app_end_remote_input_slot();
      }
      drawFrame();
      void flushHostEffects({ allowPrivilegedAction: false });
      return true;
    }
    if (beginScopedInput) {
      wasm.app_end_remote_input_slot();
    }
    return false;
  },
});

if (E2E_MODE) {
  window.__MAZOCARTA_E2E__ = {
    async waitReady() {
      const support = await e2eSupportPromise;
      return support?.waitReady?.();
    },
    isReady() {
      return false;
    },
  };
  e2eSupportPromise = import("./e2e-harness.js").then(({ installE2EHarness }) => {
    e2eSupport = installE2EHarness({
      fixtureName: SEARCH_PARAMS.get("fixture") || "",
      multiplayer,
      getWasm: () => wasm,
      getLogicalSize: () => ({ width: logicalWidth, height: logicalHeight }),
      getRunSnapshotRaw: () => lastRunSaveRaw,
      getPartySnapshot: () => lastPartySnapshot,
      getRunSaveGeneration: () => lastRunSaveGeneration,
      resetSnapshotGenerations: () => {
        lastRunSaveGeneration = -1;
        lastPartySnapshotGeneration = -1;
      },
      getBlockingScreenActionRect: () =>
        blockingScreenActionRect(multiplayer.currentBlockingScreen()),
      getBlockingScreenBannerRect: () =>
        blockingScreenBannerRect(multiplayer.currentBlockingScreen()),
      syncRunSaveSnapshot,
      syncPartySnapshot,
      drawFrame,
      flushHostEffects,
      restoreRunSnapshotRaw,
      writeStoredRun,
    });
    return e2eSupport;
  });
}

function syncStoredRunAvailability() {
  if (!wasm || typeof wasm.app_set_saved_run_available !== "function") {
    return;
  }
  wasm.app_set_saved_run_available(readStoredRun() ? 1 : 0);
}

function syncInstallCapability() {
  if (!wasm || typeof wasm.app_set_install_capability !== "function") {
    return false;
  }
  wasm.app_set_install_capability(resolveInstallCapabilityCode());
  return true;
}

function syncUpdateAvailability() {
  if (!wasm || typeof wasm.app_set_update_available !== "function") {
    return false;
  }
  const available = waitingServiceWorker && window.navigator.serviceWorker?.controller ? 1 : 0;
  wasm.app_set_update_available(available);
  return true;
}

function setWaitingServiceWorker(worker) {
  const nextWaitingWorker = window.navigator.serviceWorker?.controller ? worker : null;
  const changed = waitingServiceWorker !== nextWaitingWorker;
  waitingServiceWorker = nextWaitingWorker;
  syncUpdateAvailability();
  if (changed && wasm) {
    drawFrame();
  }
  return changed;
}

function syncWaitingServiceWorker(registration = serviceWorkerRegistration) {
  setWaitingServiceWorker(registration?.waiting ?? null);
}

function watchInstallingServiceWorker(worker, registration) {
  if (!worker) {
    return;
  }
  worker.addEventListener("statechange", () => {
    if (worker.state === "installed" || worker.state === "redundant") {
      syncWaitingServiceWorker(registration);
    }
  });
}

function syncBootScreenState() {
  if (!wasm || typeof wasm.app_is_boot_screen !== "function") {
    return false;
  }

  const bootScreenVisible = !!wasm.app_is_boot_screen();
  const enteredBoot = bootScreenVisible && lastBootScreenVisible === false;
  lastBootScreenVisible = bootScreenVisible;

  if (!enteredBoot) {
    return false;
  }

  syncWaitingServiceWorker();
  if (serviceWorkerRegistration) {
    void serviceWorkerRegistration
      .update()
      .then(() => syncWaitingServiceWorker(serviceWorkerRegistration))
      .catch(console.error);
  }
  return true;
}

function syncRunSaveSnapshot() {
  if (
    !wasm ||
    typeof wasm.run_save_generation !== "function" ||
    typeof wasm.run_save_len !== "function" ||
    typeof wasm.run_save_ptr !== "function"
  ) {
    return false;
  }

  const generation = wasm.run_save_generation();
  if (generation === lastRunSaveGeneration) {
    return false;
  }
  lastRunSaveGeneration = generation;

  const len = wasm.run_save_len();
  if (!len) {
    lastRunSaveRaw = null;
    clearStoredRun();
    syncStoredRunAvailability();
    return true;
  }

  const ptr = wasm.run_save_ptr();
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
  const raw = decoder.decode(bytes.slice());
  lastRunSaveRaw = raw;
  if (!writeStoredRun(raw)) {
    lastRunSaveRaw = null;
    clearStoredRun();
    if (typeof wasm.app_set_saved_run_available === "function") {
      wasm.app_set_saved_run_available(0);
    }
    return false;
  }

  syncStoredRunAvailability();
  if (
    multiplayer &&
    typeof multiplayer.notifyLocalRunSnapshotChanged === "function"
  ) {
    multiplayer.notifyLocalRunSnapshotChanged();
  }
  return true;
}

function rewriteSnapshotForGuest(raw, slot) {
  try {
    const payload = JSON.parse(raw);
    if (payload?.party && Number.isInteger(slot)) {
      payload.party.local_slot = slot;
      if (Array.isArray(payload.party.slots)) {
        payload.party.slots.forEach((slotState, index) => {
          if (!slotState || typeof slotState !== "object") {
            return;
          }
          if (index === slot) {
            slotState.claim = "local";
            slotState.connected = true;
          } else if (slotState.claim === "local") {
            slotState.claim = "remote";
          }
        });
      }
    }
    return JSON.stringify(payload);
  } catch (error) {
    console.error(error);
    return raw;
  }
}

function restoreRunSnapshotRaw(raw, { useMultiplayerRestore = false } = {}) {
  if (
    !wasm ||
    typeof wasm.prepare_restore_buffer !== "function" ||
    typeof wasm.app_restore_from_buffer !== "function"
  ) {
    return false;
  }

  const bytes = encoder.encode(raw);
  const ptr = wasm.prepare_restore_buffer(bytes.length);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
  const restored =
    useMultiplayerRestore && typeof wasm.app_apply_multiplayer_snapshot_from_buffer === "function"
      ? wasm.app_apply_multiplayer_snapshot_from_buffer(bytes.length)
      : wasm.app_restore_from_buffer(bytes.length);
  if (restored) {
    syncRunSaveSnapshot();
    syncPartySnapshot();
  }
  return !!restored;
}

function restoreGuestRunSnapshot(raw, slot) {
  if (typeof raw !== "string" || !raw.length) {
    return false;
  }
  return restoreRunSnapshotRaw(rewriteSnapshotForGuest(raw, slot), {
    useMultiplayerRestore: true,
  });
}

function writePairingBuffer(raw) {
  if (!wasm || typeof wasm.prepare_pairing_buffer !== "function") {
    return -1;
  }
  const bytes = encoder.encode(String(raw ?? ""));
  const ptr = wasm.prepare_pairing_buffer(bytes.length);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
  return bytes.length;
}

function readWasmUtf8Buffer(ptr, len) {
  if (!wasm || !Number.isInteger(len) || len < 0 || !Number.isInteger(ptr) || ptr < 0) {
    return "";
  }
  return decoder.decode(new Uint8Array(wasm.memory.buffer, ptr, len).slice());
}

function readPairingOutput() {
  if (
    !wasm ||
    typeof wasm.pairing_output_ptr !== "function" ||
    typeof wasm.pairing_output_len !== "function"
  ) {
    return "";
  }
  return readWasmUtf8Buffer(wasm.pairing_output_ptr(), wasm.pairing_output_len());
}

function readPairingDecodedPayload() {
  if (
    !wasm ||
    typeof wasm.pairing_decoded_payload_ptr !== "function" ||
    typeof wasm.pairing_decoded_payload_len !== "function"
  ) {
    return "";
  }
  return readWasmUtf8Buffer(
    wasm.pairing_decoded_payload_ptr(),
    wasm.pairing_decoded_payload_len(),
  );
}

function encodePairPayloadRust(payload) {
  if (
    !wasm ||
    typeof wasm.pairing_encode_payload_from_buffer !== "function"
  ) {
    return null;
  }
  const len = writePairingBuffer(JSON.stringify(payload));
  if (len < 0 || !wasm.pairing_encode_payload_from_buffer(len)) {
    return null;
  }
  return readPairingOutput();
}

function buildPairTransportFramesRust(raw, chunkChars = 96) {
  if (
    !wasm ||
    typeof wasm.pairing_build_transport_frames_from_buffer !== "function" ||
    typeof wasm.pairing_transport_frame_count !== "function" ||
    typeof wasm.pairing_export_transport_frame !== "function"
  ) {
    return null;
  }
  const len = writePairingBuffer(raw);
  if (len < 0) {
    return null;
  }
  const count = wasm.pairing_build_transport_frames_from_buffer(
    len,
    Math.max(1, Math.round(chunkChars)),
  );
  if (!Number.isInteger(count) || count <= 0) {
    return [];
  }
  const frames = [];
  for (let index = 0; index < count; index += 1) {
    if (!wasm.pairing_export_transport_frame(index)) {
      return null;
    }
    frames.push(readPairingOutput());
  }
  return frames;
}

function resetPairTransportAssemblyRust() {
  if (
    wasm &&
    typeof wasm.pairing_reset_transport_assembly === "function"
  ) {
    wasm.pairing_reset_transport_assembly();
  }
}

function consumePairTransportTextRust(raw) {
  if (
    !wasm ||
    typeof wasm.pairing_submit_transport_text_from_buffer !== "function"
  ) {
    return null;
  }
  const len = writePairingBuffer(raw);
  if (len < 0) {
    return null;
  }
  const status = wasm.pairing_submit_transport_text_from_buffer(len);
  if (status === 0) {
    throw new Error("Invalid pairing code.");
  }
  if (status === 2) {
    return {
      status: "partial",
      received:
        typeof wasm.pairing_transport_received_parts === "function"
          ? wasm.pairing_transport_received_parts()
          : 0,
      total:
        typeof wasm.pairing_transport_total_parts === "function"
          ? wasm.pairing_transport_total_parts()
          : 0,
    };
  }
  const fullCode = readPairingOutput();
  const decodedPayloadRaw = readPairingDecodedPayload();
  return {
    status: "complete",
    fullCode,
    payload: decodedPayloadRaw ? JSON.parse(decodedPayloadRaw) : null,
  };
}

function syncStoredLanguage() {
  if (
    !wasm ||
    typeof wasm.app_language_generation !== "function" ||
    typeof wasm.app_language_code !== "function"
  ) {
    return false;
  }

  const generation = wasm.app_language_generation();
  if (generation === lastLanguageGeneration) {
    return false;
  }
  lastLanguageGeneration = generation;
  return writeStoredLanguage(wasm.app_language_code());
}

function syncStoredBackgroundMode() {
  if (
    !wasm ||
    typeof wasm.app_background_mode_generation !== "function" ||
    typeof wasm.app_background_mode_code !== "function"
  ) {
    return false;
  }

  const generation = wasm.app_background_mode_generation();
  if (generation === lastBackgroundModeGeneration) {
    return false;
  }
  lastBackgroundModeGeneration = generation;
  return writeStoredBackgroundMode(wasm.app_background_mode_code());
}

function syncStoredPlayerName() {
  if (!wasm || typeof wasm.app_player_name_generation !== "function") {
    return false;
  }

  const generation = wasm.app_player_name_generation();
  if (generation === lastPlayerNameGeneration) {
    return false;
  }
  lastPlayerNameGeneration = generation;
  return writeStoredPlayerName(readAppPlayerName());
}

function currentPlayerNamePlaceholder() {
  if (!wasm || typeof wasm.app_language_code !== "function") {
    return "Player";
  }
  return wasm.app_language_code() === 1 ? "Jugador" : "Player";
}

function hidePlayerNameInput() {
  if (!playerNameInputVisible) {
    playerNameInput.style.display = "none";
    playerNameOverlay.style.display = "none";
    hideTypingPointerOverlay();
    return;
  }

  playerNameInputVisible = false;
  playerNameEditingActive = false;
  playerNameInput.style.display = "none";
  playerNameOverlay.style.display = "none";
  playerNameOverlay.classList.remove("is-focused");
  hideTypingPointerOverlay();
  setAppPlayerNameInputFocused(false);
  if (document.activeElement === playerNameInput) {
    playerNameInput.blur();
  }
}

function syncPlayerNameOverlay() {
  playerNameOverlay.style.display = "none";
}

function updateLastMouseClientPoint(event) {
  if (event.pointerType !== "mouse") {
    return;
  }
  lastMouseClientPoint = {
    x: event.clientX,
    y: event.clientY,
  };
  if (playerNameEditingActive && document.activeElement !== playerNameInput) {
    showTypingPointerOverlay();
  }
}

function hideTypingPointerOverlay() {
  typingPointerOverlay.style.display = "none";
}

function showTypingPointerOverlay() {
  if (
    !playerNameEditingActive ||
    document.activeElement === playerNameInput ||
    !lastMouseClientPoint
  ) {
    hideTypingPointerOverlay();
    return;
  }

  typingPointerOverlay.style.left = `${lastMouseClientPoint.x}px`;
  typingPointerOverlay.style.top = `${lastMouseClientPoint.y}px`;
  typingPointerOverlay.style.display = "block";
}

function playerNameInputHitTest(point) {
  if (
    !wasm ||
    typeof wasm.app_settings_player_name_input_visible !== "function" ||
    typeof wasm.app_settings_player_name_input_x !== "function" ||
    typeof wasm.app_settings_player_name_input_y !== "function" ||
    typeof wasm.app_settings_player_name_input_w !== "function" ||
    typeof wasm.app_settings_player_name_input_h !== "function"
  ) {
    return false;
  }

  if (!wasm.app_settings_player_name_input_visible()) {
    return false;
  }

  const x = wasm.app_settings_player_name_input_x();
  const y = wasm.app_settings_player_name_input_y();
  const w = wasm.app_settings_player_name_input_w();
  const h = wasm.app_settings_player_name_input_h();
  return point.x >= x && point.x <= x + w && point.y >= y && point.y <= y + h;
}

function focusHiddenPlayerNameInput() {
  setPlayerNameEditingActive(true);
  hideTypingPointerOverlay();
  if (document.activeElement !== playerNameInput) {
    playerNameInput.focus();
  }
  const end = playerNameInput.value.length;
  playerNameInput.setSelectionRange(end, end);
  setAppPlayerNameInputFocused(true);
  syncPlayerNameOverlay();
}

function syncPlayerNameInput() {
  if (
    !wasm ||
    typeof wasm.app_settings_player_name_input_visible !== "function" ||
    typeof wasm.app_settings_player_name_input_x !== "function" ||
    typeof wasm.app_settings_player_name_input_y !== "function" ||
    typeof wasm.app_settings_player_name_input_w !== "function" ||
    typeof wasm.app_settings_player_name_input_h !== "function"
  ) {
    hidePlayerNameInput();
    return false;
  }

  const visible = !!wasm.app_settings_player_name_input_visible();
  if (!visible) {
    hidePlayerNameInput();
    return false;
  }

  const nextValue = readAppPlayerName();
  const becameVisible = !playerNameInputVisible;
  const isFocused = document.activeElement === playerNameInput;
  playerNameInputVisible = true;

  if (!isFocused && !playerNameEditingActive && playerNameInput.value !== nextValue) {
    playerNameInput.value = nextValue;
  }

  playerNameInput.placeholder = currentPlayerNamePlaceholder();
  playerNameInput.style.display = "block";
  playerNameInput.style.left = "0px";
  playerNameInput.style.top = "0px";
  playerNameInput.style.width = "1px";
  playerNameInput.style.height = "1px";
  playerNameInput.style.fontSize = "16px";
  playerNameInput.style.lineHeight = "1";
  syncPlayerNameOverlay();

  if (becameVisible) {
    syncPlayerNameOverlay();
  }

  return true;
}

async function flushInstallRequest({ allowPrivilegedAction = false } = {}) {
  if (
    !wasm ||
    typeof wasm.install_request_pending !== "function" ||
    typeof wasm.clear_install_request !== "function"
  ) {
    return false;
  }

  if (!wasm.install_request_pending()) {
    return false;
  }

  if (!allowPrivilegedAction) {
    return false;
  }

  wasm.clear_install_request();
  if (!deferredInstallPrompt || typeof deferredInstallPrompt.prompt !== "function") {
    syncInstallCapability();
    return true;
  }

  try {
    await deferredInstallPrompt.prompt();
    if (deferredInstallPrompt.userChoice) {
      await deferredInstallPrompt.userChoice;
    }
  } catch (error) {
    console.error(error);
  }

  deferredInstallPrompt = null;
  syncInstallCapability();
  return true;
}

async function flushUpdateRequest({ allowPrivilegedAction = false } = {}) {
  if (
    !wasm ||
    typeof wasm.update_request_pending !== "function" ||
    typeof wasm.clear_update_request !== "function"
  ) {
    return false;
  }

  if (!wasm.update_request_pending()) {
    return false;
  }

  if (!allowPrivilegedAction) {
    return false;
  }

  wasm.clear_update_request();
  if (!waitingServiceWorker) {
    syncWaitingServiceWorker();
    return true;
  }

  reloadOnNextControllerChange = true;
  try {
    waitingServiceWorker.postMessage({ type: "SKIP_WAITING" });
  } catch (error) {
    reloadOnNextControllerChange = false;
    console.error(error);
    syncWaitingServiceWorker();
  }
  return true;
}

function flushResumeRequest() {
  if (
    !wasm ||
    typeof wasm.resume_request_pending !== "function" ||
    typeof wasm.clear_resume_request !== "function" ||
    typeof wasm.prepare_restore_buffer !== "function" ||
    typeof wasm.app_restore_from_buffer !== "function"
  ) {
    return false;
  }

  if (!wasm.resume_request_pending()) {
    return false;
  }

  const raw = readStoredRun();
  if (!raw) {
    wasm.clear_resume_request();
    syncStoredRunAvailability();
    return true;
  }

  const bytes = encoder.encode(raw);
  const ptr = wasm.prepare_restore_buffer(bytes.length);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
  const restored = wasm.app_restore_from_buffer(bytes.length);
  wasm.clear_resume_request();

  if (!restored) {
    clearStoredRun();
    syncStoredRunAvailability();
    return true;
  }

  syncRunSaveSnapshot();
  return true;
}

async function flushMultiplayerRequest() {
  if (
    !wasm ||
    typeof wasm.multiplayer_request_pending !== "function" ||
    typeof wasm.clear_multiplayer_request !== "function"
  ) {
    return false;
  }

  if (!wasm.multiplayer_request_pending()) {
    return false;
  }

  wasm.clear_multiplayer_request();
  await multiplayer.openEntryFlow();
  return true;
}

async function flushHostEffects(options = { allowPrivilegedAction: false }) {
  const installHandled = await flushInstallRequest(options);
  const updateHandled = await flushUpdateRequest(options);
  syncStoredLanguage();
  syncStoredBackgroundMode();
  syncStoredPlayerName();
  syncRunSaveSnapshot();
  const resumed = flushResumeRequest();
  const multiplayerOpened = await flushMultiplayerRequest();
  if (installHandled || updateHandled || resumed || multiplayerOpened) {
    drawFrame();
  }
  await flushShareRequest();
}

function takeShareRequest() {
  if (
    !wasm ||
    typeof wasm.share_request_len !== "function" ||
    typeof wasm.share_request_ptr !== "function" ||
    typeof wasm.clear_share_request !== "function"
  ) {
    return null;
  }
  const len = wasm.share_request_len();
  if (!len) {
    return null;
  }
  const ptr = wasm.share_request_ptr();
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
  const text = decoder.decode(bytes.slice());
  wasm.clear_share_request();
  return text;
}

function parseShareRequest(raw) {
  try {
    const data = JSON.parse(raw);
    if (
      data &&
      data.kind === "final_victory_card" &&
      typeof data.max_hp === "number" &&
      typeof data.deck_size === "number" &&
      typeof data.seed === "string" &&
      typeof data.version === "string"
    ) {
      return {
        kind: data.kind,
        title: typeof data.title === "string" ? data.title : GAME_TITLE,
        maxHp: Math.max(0, Math.round(data.max_hp)),
        deckSize: Math.max(0, Math.round(data.deck_size)),
        seed: data.seed.toUpperCase(),
        version: data.version,
        shareText: typeof data.share_text === "string" ? data.share_text : GAME_TITLE,
      };
    }
  } catch {}

  return {
    kind: "legacy_text",
    title: GAME_TITLE,
    shareText: raw,
  };
}

function formatShareDate(date = new Date()) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function formatShareFileStamp(date = new Date()) {
  const months = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
  const year = date.getFullYear();
  const month = months[date.getMonth()] ?? "jan";
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}${month}${day}`;
}

function formatShareCaption(payload, pageUrl, dateLabel) {
  return [payload.shareText, `v${payload.version}`, dateLabel, pageUrl].join(" • ");
}

function fitCanvasTextSize(context, text, desiredSize, maxWidth, weight = 700) {
  let size = desiredSize;
  while (size > 12) {
    context.font = `${weight} ${size}px ${MONO_STACK}`;
    if (context.measureText(text).width <= maxWidth) {
      return size;
    }
    size -= 1;
  }
  return 12;
}

function takeShareCaptureRect() {
  if (
    !wasm ||
    typeof wasm.share_capture_x !== "function" ||
    typeof wasm.share_capture_y !== "function" ||
    typeof wasm.share_capture_w !== "function" ||
    typeof wasm.share_capture_h !== "function"
  ) {
    return null;
  }

  const rect = {
    x: wasm.share_capture_x(),
    y: wasm.share_capture_y(),
    w: wasm.share_capture_w(),
    h: wasm.share_capture_h(),
  };
  if (!(rect.w > 0 && rect.h > 0)) {
    return null;
  }
  return rect;
}

function captureCanvasRegion(rect) {
  const transform = viewportTransform();
  const sx = rect.x * transform.scaleX + transform.offsetX;
  const sy = rect.y * transform.scaleY + transform.offsetY;
  const sw = rect.w * transform.scaleX;
  const sh = rect.h * transform.scaleY;
  const width = Math.max(1, Math.round(sw));
  const height = Math.max(1, Math.round(sh));
  const cropCanvas = document.createElement("canvas");
  cropCanvas.width = width;
  cropCanvas.height = height;
  const cropCtx = cropCanvas.getContext("2d");
  cropCtx.drawImage(canvas, sx, sy, sw, sh, 0, 0, width, height);
  return cropCanvas;
}

function canvasToBlob(canvasEl) {
  return new Promise((resolve, reject) => {
    canvasEl.toBlob((blob) => {
      if (blob) {
        resolve(blob);
        return;
      }
      reject(new Error("Failed to encode share card."));
    }, "image/png");
  });
}

async function copyShareCardToClipboard(blob, caption) {
  if (!navigator.clipboard?.write || typeof ClipboardItem === "undefined") {
    return false;
  }

  try {
    await navigator.clipboard.write([
      new ClipboardItem({
        "image/png": blob,
        "text/plain": new Blob([caption], { type: "text/plain" }),
      }),
    ]);
    console.info("Share card copied to clipboard.");
    return true;
  } catch {
    return false;
  }
}

function downloadBlob(blob, fileName) {
  const objectUrl = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = objectUrl;
  link.download = fileName;
  link.rel = "noopener";
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(objectUrl), 1000);
}

async function flushLegacyShareRequest(text) {
  const url = window.location.href;
  try {
    if (navigator.share) {
      await navigator.share({
        title: GAME_TITLE,
        text,
        url,
      });
      return;
    }

    const payload = `${text} ${url}`.trim();
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(payload);
      console.info("Share text copied to clipboard.");
      return;
    }
    console.warn("No share target available for legacy share request.");
  } catch (error) {
    if (error?.name !== "AbortError") {
      console.error(error);
    }
  }
}

async function flushShareRequest() {
  const raw = takeShareRequest();
  if (!raw) {
    return;
  }

  const payload = parseShareRequest(raw);
  if (payload.kind !== "final_victory_card") {
    await flushLegacyShareRequest(payload.shareText);
    return;
  }

  try {
    const dateLabel = formatShareDate();
    const fileStamp = formatShareFileStamp();
    const captureRect = takeShareCaptureRect();
    const imageCanvas = captureRect ? captureCanvasRegion(captureRect) : canvas;
    const blob = await canvasToBlob(imageCanvas);
    const fileName = `mazocarta-${fileStamp}.png`;
    const caption = formatShareCaption(payload, window.location.href, dateLabel);
    const file =
      typeof File === "function"
        ? new File([blob], fileName, { type: "image/png" })
        : null;

    if (file && navigator.share) {
      const shareData = {
        title: payload.title,
        text: caption,
        files: [file],
      };
      let canShareFiles = true;
      if (typeof navigator.canShare === "function") {
        canShareFiles = navigator.canShare(shareData);
      }
      if (canShareFiles) {
        try {
          await navigator.share(shareData);
          return;
        } catch (error) {
          if (error?.name === "AbortError") {
            return;
          }
          console.error(error);
        }
      }
    }

    const copiedImage = await copyShareCardToClipboard(blob, caption);
    if (!copiedImage && navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(caption);
    }
    downloadBlob(blob, fileName);
  } catch (error) {
    if (error?.name !== "AbortError") {
      console.error(error);
    }
  }
}

function onPointerMove(event) {
  if (!wasm) {
    return;
  }
  if (multiplayer.shouldBlockGameplayInput()) {
    hidePlayerNameInput();
    return;
  }
  if (multiplayer.handleGuestPointerMove(normalizedCanvasPoint(event))) {
    hidePlayerNameInput();
    return;
  }
  if (event.pointerType === "touch") {
    clearHover();
    hideTypingPointerOverlay();
    drawFrame();
    return;
  }
  updateLastMouseClientPoint(event);
  if (playerNameEditingActive) {
    showTypingPointerOverlay();
  } else {
    hideTypingPointerOverlay();
  }
  const point = toCanvasPoint(event);
  wasm.pointer_move(point.x, point.y);
  drawFrame();
}

function onPointerDown(event) {
  if (!wasm) {
    return;
  }
  event.preventDefault();
  if (multiplayer.isRoomOpen()) {
    hidePlayerNameInput();
    return;
  }
  const normalizedPoint = normalizedCanvasPoint(event);
  const blocking = multiplayer.currentBlockingScreen();
  if (blocking) {
    const point = toCanvasPoint(event);
    if (pointInRect(point, blockingScreenActionRect(blocking)) && blocking.action) {
      multiplayer.activateBlockingAction?.(blocking.action);
    }
    if (blocking.presentation !== "banner") {
      hidePlayerNameInput();
      return;
    }
    if (multiplayer.handleGuestPointerDown(normalizedPoint)) {
      if (playerNameEditingActive) {
        setPlayerNameEditingActive(false);
      }
      hidePlayerNameInput();
      return;
    }
  } else if (multiplayer.handleGuestPointerDown(normalizedPoint)) {
    if (playerNameEditingActive) {
      setPlayerNameEditingActive(false);
    }
    hidePlayerNameInput();
    return;
  }
  mixEntropy();
  const point = toCanvasPoint(event);
  if (event.pointerType === "touch") {
    clearHover();
    hideTypingPointerOverlay();
  } else {
    updateLastMouseClientPoint(event);
    if (playerNameEditingActive) {
      showTypingPointerOverlay();
    } else {
      hideTypingPointerOverlay();
    }
  }
  if (playerNameInputHitTest(point)) {
    lastPointerDownStartedOnPlayerNameInput = true;
    if (event.pointerType === "touch") {
      focusHiddenPlayerNameInput();
    } else {
      setPlayerNameEditingActive(true);
    }
    drawFrame();
    return;
  }
  if (playerNameEditingActive) {
    setPlayerNameEditingActive(false);
  }
  lastPointerDownStartedOnPlayerNameInput = false;
  wasm.pointer_down(point.x, point.y);
  drawFrame();
  void flushHostEffects({ allowPrivilegedAction: false });
}

function onPointerUp(event) {
  if (!wasm) {
    return;
  }
  if (multiplayer.shouldBlockGameplayInput()) {
    hidePlayerNameInput();
    return;
  }
  if (multiplayer.handleGuestPointerUp(normalizedCanvasPoint(event))) {
    hidePlayerNameInput();
    return;
  }
  if (event.pointerType === "mouse") {
    updateLastMouseClientPoint(event);
    if (playerNameEditingActive) {
      showTypingPointerOverlay();
    } else {
      hideTypingPointerOverlay();
    }
  }
  const point = toCanvasPoint(event);
  const pointerDownStartedOnPlayerNameInput = lastPointerDownStartedOnPlayerNameInput;
  lastPointerDownStartedOnPlayerNameInput = false;
  if (pointerDownStartedOnPlayerNameInput) {
    if (event.pointerType === "touch") {
      clearHover();
      hideTypingPointerOverlay();
    }
    drawFrame();
    return;
  }
  wasm.pointer_up(point.x, point.y);
  if (playerNameInputHitTest(point)) {
    if (event.pointerType === "touch") {
      clearHover();
      hideTypingPointerOverlay();
    }
    drawFrame();
    return;
  }
  if (event.pointerType === "touch") {
    clearHover();
  }
  drawFrame();
  void flushHostEffects({ allowPrivilegedAction: true });
}

function onPointerCancel() {
  lastPointerDownStartedOnPlayerNameInput = false;
  clearHover();
  hideTypingPointerOverlay();
  drawFrame();
}

function onPointerLeave() {
  clearHover();
  hideTypingPointerOverlay();
  drawFrame();
}

function keyCodeFor(event) {
  if (event.key === "Enter") {
    return 13;
  }
  if (event.key === " ") {
    return 32;
  }
  if (event.key === "Escape") {
    return 27;
  }
  if (/^[1-9]$/.test(event.key)) {
    return event.key.charCodeAt(0);
  }
  if (/^[a-z]$/i.test(event.key)) {
    return event.key.toUpperCase().charCodeAt(0);
  }
  return null;
}

function activeElementAcceptsTextInput() {
  const active = document.activeElement;
  if (
    active instanceof HTMLInputElement ||
    active instanceof HTMLTextAreaElement ||
    active instanceof HTMLSelectElement
  ) {
    return true;
  }
  return !!active?.isContentEditable;
}

function onKeyDown(event) {
  if (!wasm) {
    return;
  }
  if (multiplayer.shouldBlockGameplayInput() && !activeElementAcceptsTextInput()) {
    return;
  }
  if (event.ctrlKey || event.metaKey || event.altKey) {
    return;
  }
  if (handlePlayerNameEditingKey(event)) {
    return;
  }
  if (document.activeElement === playerNameInput || activeElementAcceptsTextInput()) {
    return;
  }
  const code = keyCodeFor(event);
  if (code == null) {
    return;
  }
  event.preventDefault();
  if (multiplayer.handleGuestKeyDown(code)) {
    hidePlayerNameInput();
    return;
  }
  mixEntropy();
  wasm.key_down(code);
  drawFrame();
  void flushHostEffects({ allowPrivilegedAction: true });
}

function onPlayerNameInput(event) {
  if (!wasm) {
    return;
  }

  const limitedValue = limitPlayerNameInputValue(playerNameInput.value);
  playerNameInput.value = limitedValue;
  setAppPlayerName(limitedValue);
  syncStoredPlayerName();
  syncPlayerNameOverlay();
  drawFrame();
}

function onPlayerNameInputKeyDown(event) {
  event.stopPropagation();
  if (!wasm) {
    return;
  }

  if (event.key === "Escape") {
    event.preventDefault();
    setPlayerNameEditingActive(false);
    drawFrame();
    return;
  }

  if (event.key === "Enter") {
    event.preventDefault();
    setPlayerNameEditingActive(false);
    drawFrame();
  }
}

function syncPlayerNameSelection(event) {
  event.stopPropagation();
  syncPlayerNameOverlay();
}

function onPlayerNameInputFocus(event) {
  event.stopPropagation();
  setPlayerNameEditingActive(true);
  drawFrame();
}

function onPlayerNameInputBlur(event) {
  event.stopPropagation();
  setPlayerNameEditingActive(false);
  drawFrame();
}

async function registerServiceWorker() {
  if (!("serviceWorker" in window.navigator)) {
    return;
  }

  if (isAndroidAppHost()) {
    try {
      const registrations = await window.navigator.serviceWorker.getRegistrations();
      await Promise.all(registrations.map((registration) => registration.unregister()));
    } catch (error) {
      console.error(error);
    }
    return;
  }

  if (E2E_MODE) {
    try {
      const registrations = await window.navigator.serviceWorker.getRegistrations();
      await Promise.all(registrations.map((registration) => registration.unregister()));
    } catch (error) {
      console.error(error);
    }
    return;
  }

  if (isLocalDevHost()) {
    try {
      const registrations = await window.navigator.serviceWorker.getRegistrations();
      await Promise.all(registrations.map((registration) => registration.unregister()));
      const cacheKeys = await caches.keys();
      await Promise.all(
        cacheKeys
          .filter((key) => key.startsWith("mazocarta-shell-"))
          .map((key) => caches.delete(key)),
      );
    } catch (error) {
      console.error(error);
    }
    return;
  }

  try {
    const registration = await window.navigator.serviceWorker.register("./sw.js", {
      scope: "./",
      updateViaCache: "none",
    });
    serviceWorkerRegistration = registration;
    syncWaitingServiceWorker(registration);
    if (registration.installing) {
      watchInstallingServiceWorker(registration.installing, registration);
    }
    registration.addEventListener("updatefound", () => {
      watchInstallingServiceWorker(registration.installing, registration);
    });
    window.navigator.serviceWorker.addEventListener("controllerchange", () => {
      if (reloadOnNextControllerChange) {
        reloadOnNextControllerChange = false;
        window.location.reload();
        return;
      }
      syncWaitingServiceWorker(registration);
      if (wasm) {
        drawFrame();
      }
    });
    try {
      await registration.update();
    } catch (error) {
      console.error(error);
    }
    syncWaitingServiceWorker(registration);
  } catch (error) {
    console.error(error);
  }
}

async function loadWasm() {
  resizeCanvas();
  try {
    void preloadShellImages();
    const debugEnabled = await resolveDebugEnabled();
    const response = await fetch("./mazocarta.wasm", { cache: "no-store" });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const bytes = await response.arrayBuffer();
    const { instance } = await WebAssembly.instantiate(bytes, {});
    wasm = instance.exports;
    mixEntropy();
    wasm.app_init(logicalWidth, logicalHeight);
    if (typeof wasm.app_set_language === "function") {
      wasm.app_set_language(readStoredLanguage());
    }
    if (typeof wasm.app_set_background_mode === "function") {
      wasm.app_set_background_mode(readStoredBackgroundMode());
    }
    setAppPlayerName(readStoredPlayerName());
    setAppPlayerNameInputFocused(false);
    if (typeof wasm.app_set_debug_mode === "function") {
      wasm.app_set_debug_mode(debugEnabled ? 1 : 0);
    }
    syncInstallCapability();
    syncWaitingServiceWorker();
    syncUpdateAvailability();
    syncStoredRunAvailability();
    lastBootScreenVisible =
      typeof wasm.app_is_boot_screen === "function" ? !!wasm.app_is_boot_screen() : null;
    lastLanguageGeneration =
      typeof wasm.app_language_generation === "function" ? wasm.app_language_generation() : 0;
    lastBackgroundModeGeneration =
      typeof wasm.app_background_mode_generation === "function"
        ? wasm.app_background_mode_generation()
        : 0;
    lastPlayerNameGeneration =
      typeof wasm.app_player_name_generation === "function" ? wasm.app_player_name_generation() : 0;
    lastRunSaveGeneration = Number.NaN;
    lastPartySnapshotGeneration = Number.NaN;
    syncRunSaveSnapshot();
    syncPartySnapshot();
    const support = await e2eSupportPromise;
    await support?.maybeLoadFixture?.();
    document.title = GAME_TITLE;
    drawFrame();
    support?.markReady?.();

    const loop = (timestamp) => {
      if (!lastFrameTime) {
        lastFrameTime = timestamp;
      }
      const dt = timestamp - lastFrameTime;
      lastFrameTime = timestamp;
      try {
        wasm.app_tick(dt);
        drawFrame();
        e2eSupport?.incrementFrameCounter?.();
      } catch (error) {
        console.error(error);
      } finally {
        rafId = window.requestAnimationFrame(loop);
      }
    };

    rafId = window.requestAnimationFrame(loop);
  } catch (error) {
    document.title = GAME_TITLE;
    console.error(error);
    e2eSupport?.failReady?.(error);
  }
}

canvas.addEventListener("pointermove", onPointerMove);
canvas.addEventListener("pointerdown", onPointerDown);
canvas.addEventListener("pointerup", onPointerUp);
canvas.addEventListener("pointercancel", onPointerCancel);
canvas.addEventListener("pointerleave", onPointerLeave);
playerNameInput.addEventListener("input", onPlayerNameInput);
playerNameInput.addEventListener("keydown", onPlayerNameInputKeyDown);
playerNameInput.addEventListener("keyup", syncPlayerNameSelection);
playerNameInput.addEventListener("focus", onPlayerNameInputFocus);
playerNameInput.addEventListener("blur", onPlayerNameInputBlur);
playerNameInput.addEventListener("select", syncPlayerNameOverlay);
window.addEventListener("beforeinstallprompt", (event) => {
  event.preventDefault();
  deferredInstallPrompt = event;
  syncInstallCapability();
  drawFrame();
});
window.addEventListener("appinstalled", () => {
  deferredInstallPrompt = null;
  syncInstallCapability();
  drawFrame();
});
window.addEventListener("keydown", onKeyDown);
window.addEventListener("resize", resizeCanvas);
window
  .matchMedia?.("(display-mode: standalone)")
  ?.addEventListener?.("change", () => {
    syncInstallCapability();
    drawFrame();
  });
window.addEventListener("pagehide", () => {
  syncStoredLanguage();
  syncStoredBackgroundMode();
  syncStoredPlayerName();
  syncRunSaveSnapshot();
});
window.addEventListener("pageshow", () => {
  syncInstallCapability();
  syncWaitingServiceWorker();
  if (serviceWorkerRegistration) {
    void serviceWorkerRegistration.update().catch(console.error);
  }
  drawFrame();
});
window.addEventListener("beforeunload", () => {
  void multiplayer.destroy();
  syncStoredLanguage();
  syncStoredBackgroundMode();
  syncStoredPlayerName();
  syncRunSaveSnapshot();
  if (rafId) {
    window.cancelAnimationFrame(rafId);
  }
});

void registerServiceWorker();
loadWasm();
