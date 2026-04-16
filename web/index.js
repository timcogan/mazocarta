const canvas = document.getElementById("game");
if (!(canvas instanceof HTMLCanvasElement)) {
  throw new Error("Canvas element `#game` not found");
}
const ctx = canvas.getContext("2d");
if (!ctx) {
  throw new Error("2D canvas context unavailable.");
}
const decoder = new TextDecoder();
const encoder = new TextEncoder();
const blurCanvas = document.createElement("canvas");
const blurCtx = blurCanvas.getContext("2d");
const imageCache = new Map();

let wasm = null;
let rafId = 0;
let lastFrameTime = 0;
let logicalWidth = 1280;
let logicalHeight = 720;
let lastRunSaveGeneration = 0;
let lastLanguageGeneration = 0;
let deferredInstallPrompt = null;
let serviceWorkerRegistration = null;
let waitingServiceWorker = null;
let reloadOnNextControllerChange = false;
let lastBootScreenVisible = null;
const MONO_STACK =
  '"IBM Plex Mono", "JetBrains Mono", "Cascadia Mono", "Fira Code", "Liberation Mono", monospace';
const GAME_TITLE = "Mazocarta";
const LOGO_ASSET_PATH = "./mazocarta.svg";
const SHARE_CARD_WIDTH = 420;
const SHARE_CARD_HEIGHT = 500;
const ACTIVE_RUN_STORAGE_KEY = "mazocarta.active_run";
const LANGUAGE_STORAGE_KEY = "mazocarta.language";

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

function resolveInstallCapabilityCode() {
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
  const value = new URLSearchParams(window.location.search).get("debug");
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
    default:
      return `400 ${size}px ${MONO_STACK}`;
  }
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

function drawFrame() {
  if (!wasm) {
    return;
  }

  syncBootScreenState();

  const ptr = wasm.frame_ptr();
  const len = wasm.frame_len();
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
  const scene = decoder.decode(bytes);
  const transform = viewportTransform();

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
      ctx.fillRect(0, 0, logicalWidth, logicalHeight);
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
        const centerX = logicalWidth * 0.5;
        const centerY = logicalHeight * 0.5;
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
      ctx.fillText(text, Number.parseFloat(x), Number.parseFloat(y));
    }
  }

  ctx.restore();
}

function toCanvasPoint(event) {
  const rect = canvas.getBoundingClientRect();
  return {
    x: (event.clientX - rect.left) * (logicalWidth / rect.width),
    y: (event.clientY - rect.top) * (logicalHeight / rect.height),
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
    clearStoredRun();
    syncStoredRunAvailability();
    return true;
  }

  const ptr = wasm.run_save_ptr();
  const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
  const raw = decoder.decode(bytes.slice());
  if (!writeStoredRun(raw)) {
    clearStoredRun();
    if (typeof wasm.app_set_saved_run_available === "function") {
      wasm.app_set_saved_run_available(0);
    }
    return false;
  }

  syncStoredRunAvailability();
  return true;
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

async function flushHostEffects(options = { allowPrivilegedAction: false }) {
  const installHandled = await flushInstallRequest(options);
  const updateHandled = await flushUpdateRequest(options);
  syncStoredLanguage();
  syncRunSaveSnapshot();
  const resumed = flushResumeRequest();
  if (installHandled || updateHandled || resumed) {
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
  if (event.pointerType === "touch") {
    clearHover();
    drawFrame();
    return;
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
  mixEntropy();
  const point = toCanvasPoint(event);
  if (event.pointerType === "touch") {
    clearHover();
  }
  wasm.pointer_down(point.x, point.y);
  drawFrame();
  void flushHostEffects({ allowPrivilegedAction: false });
}

function onPointerUp(event) {
  if (!wasm) {
    return;
  }
  const point = toCanvasPoint(event);
  wasm.pointer_up(point.x, point.y);
  if (event.pointerType === "touch") {
    clearHover();
  }
  drawFrame();
  void flushHostEffects({ allowPrivilegedAction: true });
}

function onPointerCancel() {
  clearHover();
  drawFrame();
}

function onPointerLeave() {
  clearHover();
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

function onKeyDown(event) {
  if (!wasm) {
    return;
  }
  const code = keyCodeFor(event);
  if (code == null) {
    return;
  }
  event.preventDefault();
  mixEntropy();
  wasm.key_down(code);
  drawFrame();
  void flushHostEffects({ allowPrivilegedAction: true });
}

async function registerServiceWorker() {
  if (!("serviceWorker" in window.navigator)) {
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
    lastRunSaveGeneration =
      typeof wasm.run_save_generation === "function" ? wasm.run_save_generation() : 0;
    document.title = "Mazocarta";
    drawFrame();

    const loop = (timestamp) => {
      if (!lastFrameTime) {
        lastFrameTime = timestamp;
      }
      const dt = timestamp - lastFrameTime;
      lastFrameTime = timestamp;
      wasm.app_tick(dt);
      drawFrame();
      rafId = window.requestAnimationFrame(loop);
    };

    rafId = window.requestAnimationFrame(loop);
  } catch (error) {
    document.title = "Mazocarta";
    console.error(error);
  }
}

canvas.addEventListener("pointermove", onPointerMove);
canvas.addEventListener("pointerdown", onPointerDown);
canvas.addEventListener("pointerup", onPointerUp);
canvas.addEventListener("pointercancel", onPointerCancel);
canvas.addEventListener("pointerleave", onPointerLeave);
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
  syncStoredLanguage();
  syncRunSaveSnapshot();
  if (rafId) {
    window.cancelAnimationFrame(rafId);
  }
});

void registerServiceWorker();
loadWasm();
