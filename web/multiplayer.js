import {
  appendChildren,
  createElement,
  uiActions,
  uiButton,
  uiButtonRow,
  uiCard,
  uiChip,
  uiCopy,
  uiHeaderCopy,
  uiShell,
  uiStack,
  uiTitle,
  uiToggleRow,
} from "./ui-kit.js";

const ICE_GATHER_TIMEOUT_MS = 4000;
const SCENE_BROADCAST_INTERVAL_MS = 50;
const SCANNER_PARTIAL_SUCCESS_COOLDOWN_MS = 90;
const SCANNER_COMPLETE_SUCCESS_COOLDOWN_MS = 180;
const QRCODE_IMPORT_URL = "./qrcode.bundle.mjs";
const PAIR_QR_CANVAS_SIZE = 448;
const PAIR_QR_FRAME_INTERVAL_MS = 350;
const ENTRY_BACK_GUARD_MS = 450;
// Lower per-frame density to improve real-camera decode reliability.
const PAIR_QR_CHUNK_PAYLOAD_CHARS = 64;
const MIN_MULTIPLAYER_PARTY_SIZE = 2;
const MAX_PARTY_SIZE = 7;
const PLAYER_BARRIER_SCREENS = new Set(["reward", "shop", "rest", "module_select", "event"]);
const PAIR_TOKEN_BYTES = 4;

function randomHex(count) {
  const bytes = new Uint8Array(count);
  if (window.crypto?.getRandomValues) {
    window.crypto.getRandomValues(bytes);
  } else {
    for (let index = 0; index < bytes.length; index += 1) {
      bytes[index] = Math.floor(Math.random() * 256);
    }
  }
  return Array.from(bytes, (value) => value.toString(16).padStart(2, "0")).join("");
}

function createPeerId() {
  return randomHex(8);
}

function sanitizeDisplayName(raw) {
  const value = typeof raw === "string" ? raw.trim() : "";
  return value.length > 0 ? value.slice(0, 12) : "Player";
}

function clampPartySize(value, minimum = MIN_MULTIPLAYER_PARTY_SIZE) {
  const normalized = Number.isFinite(value) ? Math.round(value) : MIN_MULTIPLAYER_PARTY_SIZE;
  return Math.max(minimum, Math.min(MAX_PARTY_SIZE, normalized));
}

function buildParticipantList(state, getLocalName) {
  const localParticipant = {
    peerId: state.localPeerId,
    name: sanitizeDisplayName(getLocalName()),
    connected: true,
  };
  if (state.role !== "host") {
    return [localParticipant];
  }

  const participants = [localParticipant];
  for (const peer of state.peers.values()) {
    participants.push({
      peerId: peer.peerId || peer.fallbackPeerId,
      name: sanitizeDisplayName(peer.name || "Guest"),
      connected: peer.dc?.readyState === "open",
    });
  }
  return participants;
}

function normalizePartySnapshot(snapshot) {
  if (!snapshot || typeof snapshot !== "object") {
    return null;
  }
  if (!Array.isArray(snapshot.slots)) {
    return null;
  }
  return snapshot;
}

function extractPartySnapshotFromRunSnapshot(raw, slotOverride = null) {
  try {
    const payload = JSON.parse(raw);
    const party = normalizePartySnapshot(payload?.party);
    if (!party) {
      return null;
    }
    if (Number.isInteger(slotOverride)) {
      party.local_slot = slotOverride;
    }
    return party;
  } catch (error) {
    console.error(error);
    return null;
  }
}

function normalizeSlotOption(option) {
  if (!option || !Number.isInteger(option.slot) || option.slot < 0) {
    return null;
  }
  return {
    slot: option.slot,
    kind: option.kind === "reclaim" ? "reclaim" : "open",
    name: sanitizeDisplayName(option.name || `Slot ${option.slot + 1}`),
    label:
      typeof option.label === "string" && option.label.trim().length > 0
        ? option.label.trim()
        : `Slot ${option.slot + 1}`,
    detail:
      typeof option.detail === "string" && option.detail.trim().length > 0
        ? option.detail.trim()
        : option.kind === "reclaim"
          ? "Reconnect to this live hero."
          : "Claim this open slot.",
  };
}

function normalizeSummary(summary) {
  if (!summary || !summary.inCombat) {
    return null;
  }
  return {
    inCombat: true,
    hp: Number.isFinite(summary.hp) ? Math.max(0, Math.round(summary.hp)) : 0,
    maxHp: Number.isFinite(summary.maxHp) ? Math.max(0, Math.round(summary.maxHp)) : 0,
    block: Number.isFinite(summary.block) ? Math.max(0, Math.round(summary.block)) : 0,
  };
}

function createScenePayload(scene, viewport, summary, participants, party) {
  return {
    type: "scene",
    scene,
    width: viewport.width,
    height: viewport.height,
    summary: normalizeSummary(summary),
    participants,
    party: normalizePartySnapshot(party),
  };
}

function waitForIceComplete(peerConnection) {
  if (peerConnection.iceGatheringState === "complete") {
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    const onChange = () => {
      if (peerConnection.iceGatheringState === "complete") {
        peerConnection.removeEventListener("icegatheringstatechange", onChange);
        resolve();
      }
    };
    peerConnection.addEventListener("icegatheringstatechange", onChange);
    window.setTimeout(() => {
      peerConnection.removeEventListener("icegatheringstatechange", onChange);
      resolve();
    }, ICE_GATHER_TIMEOUT_MS);
  });
}

function safeSend(channel, payload) {
  if (!channel || channel.readyState !== "open") {
    return false;
  }
  try {
    channel.send(JSON.stringify(payload));
    return true;
  } catch (error) {
    console.error(error);
    return false;
  }
}

let qrModulePromise = null;
let jsQrLoaderPromise = null;
let qrRenderToken = 0;

async function renderQr(canvas, textOrFrames) {
  if (!(canvas instanceof HTMLCanvasElement)) {
    return;
  }
  const cssSize = Math.max(1, Math.round(canvas.clientWidth || canvas.width || PAIR_QR_CANVAS_SIZE));
  const devicePixelRatio = Math.max(1, Math.min(window.devicePixelRatio || 1, 2));
  const renderSize = Math.max(1, Math.round(cssSize * devicePixelRatio));
  if (canvas.width !== renderSize || canvas.height !== renderSize) {
    canvas.width = renderSize;
    canvas.height = renderSize;
  }
  const context = canvas.getContext("2d");
  if (!context) {
    return;
  }

  context.clearRect(0, 0, canvas.width, canvas.height);
  const frames = Array.isArray(textOrFrames)
    ? textOrFrames.filter((frame) => typeof frame === "string" && frame.length > 0)
    : typeof textOrFrames === "string" && textOrFrames.length > 0
      ? [textOrFrames]
      : [];
  if (!frames.length) {
    qrRenderToken += 1;
    return;
  }

  try {
    if (!qrModulePromise) {
      qrModulePromise = import(QRCODE_IMPORT_URL);
    }
    const qrModule = await qrModulePromise;
    const token = qrRenderToken + 1;
    qrRenderToken = token;
    const renderFrame = async (index) => {
      if (qrRenderToken !== token) {
        return;
      }
      await qrModule.toCanvas(canvas, frames[index], {
        width: canvas.width,
        margin: 3,
        color: {
          dark: "#000000",
          light: "#ffffff",
        },
        errorCorrectionLevel: "L",
      });
      if (frames.length > 1 && qrRenderToken === token) {
        window.setTimeout(() => {
          if (qrRenderToken === token) {
            void renderFrame((index + 1) % frames.length);
          }
        }, PAIR_QR_FRAME_INTERVAL_MS);
      }
    };
    await renderFrame(0);
  } catch (error) {
    console.error(error);
    context.fillStyle = "#ffffff";
    context.fillRect(0, 0, canvas.width, canvas.height);
    context.fillStyle = "#000000";
    context.font = "700 18px monospace";
    context.textAlign = "center";
    context.fillText("QR unavailable", canvas.width * 0.5, canvas.height * 0.5);
  }
}

function hasCameraCaptureSupport() {
  return !!window.navigator.mediaDevices?.getUserMedia;
}

function hasNativeQrDetector() {
  return "BarcodeDetector" in window && typeof window.BarcodeDetector === "function";
}

function cameraAvailabilityMessage() {
  if (!window.isSecureContext) {
    return "Camera needs HTTPS on phones. localhost works on laptops, but LAN IP over http does not.";
  }
  if (!window.navigator.mediaDevices) {
    return "Camera APIs are missing in this browser.";
  }
  if (typeof window.navigator.mediaDevices.getUserMedia !== "function") {
    return "This browser exposes no camera capture API here.";
  }
  return "Camera access is unavailable in this browser.";
}

function defaultScannerFacingMode() {
  return "environment";
}

function scannerFacingToggleLabel(currentFacing) {
  return currentFacing === "environment" ? "Use front camera" : "Use back camera";
}

function scannerFacingStatusLabel(currentFacing) {
  return currentFacing === "environment" ? "Back camera selected." : "Front camera selected.";
}

function preferredVideoConstraints(facingMode, exact = false) {
  const facingConstraint = exact ? { exact: facingMode } : { ideal: facingMode };
  return {
    video: {
      facingMode: facingConstraint,
      width: { ideal: 1280 },
      height: { ideal: 720 },
    },
  };
}

async function ensureJsQrLoaded() {
  if (typeof window.jsQR === "function") {
    return window.jsQR;
  }
  if (!jsQrLoaderPromise) {
    jsQrLoaderPromise = new Promise((resolve, reject) => {
      const existing = document.querySelector('script[data-jsqr-loader="true"]');
      if (existing instanceof HTMLScriptElement) {
        existing.addEventListener("load", () => {
          if (typeof window.jsQR === "function") {
            resolve(window.jsQR);
          } else {
            reject(new Error("jsQR loaded without a usable export."));
          }
        });
        existing.addEventListener("error", () => reject(new Error("Could not load jsQR.")));
        return;
      }

      const script = document.createElement("script");
      script.src = "./jsqr.js";
      script.async = true;
      script.dataset.jsqrLoader = "true";
      script.addEventListener("load", () => {
        if (typeof window.jsQR === "function") {
          resolve(window.jsQR);
        } else {
          reject(new Error("jsQR loaded without a usable export."));
        }
      });
      script.addEventListener("error", () => reject(new Error("Could not load jsQR.")));
      document.head.appendChild(script);
    }).catch((error) => {
      jsQrLoaderPromise = null;
      throw error;
    });
  }
  return jsQrLoaderPromise;
}

export function createMultiplayerController(options) {
  const root = createElement("div", { id: "multiplayer-ui" });
  document.body.appendChild(root);

  const state = {
    role: "none",
    sessionFailure: null,
    localPeerId: createPeerId(),
    localSummary: null,
    localParty: null,
    remoteFrame: null,
    lastGuestRunSnapshot: "",
    peers: new Map(),
    guestConnection: null,
    lastSceneSent: "",
    lastSceneSentAt: 0,
    sessionStarted: false,
    nextJoinIndex: 1,
    partySize: clampPartySize(options.getConfiguredPartySize?.() || MIN_MULTIPLAYER_PARTY_SIZE, 1),
    room: {
      open: false,
      mode: "entry",
      entryOpenedAt: 0,
      hostRunMode: "new",
      status: "",
      note: "",
      error: "",
      transportProgress: null,
      scannerDetection: null,
      scannerPreviewAspect: null,
      scannerVideoMountId: 0,
      localCode: "",
      localQrFrames: [],
      manualInput: "",
      transportMode: "qr_loop",
      pairing: null,
      guestResumeAction: null,
      invitationId: null,
      remoteGuestName: "",
      pendingConnection: null,
      scannerActive: false,
      scannerStream: null,
      scannerToken: 0,
      outputMode: "qr",
      inputMode: "camera",
      awaitingPeerOpen: false,
      cameraFacing: defaultScannerFacingMode(),
    },
  };

  function localName() {
    return sanitizeDisplayName(options.getLocalName?.() || "Player");
  }

  function scannerSupported() {
    return hasCameraCaptureSupport();
  }

  function currentPartySnapshot() {
    if (state.role === "guest") {
      return normalizePartySnapshot(state.localParty || state.remoteFrame?.party);
    }
    return normalizePartySnapshot(state.localParty);
  }

  function currentParticipants() {
    const party = currentPartySnapshot();
    if (party) {
      return party.slots
        .filter((slot) => slot && Number.isInteger(slot.slot) && slot.slot < party.configured_party_size)
        .map((slot) => ({
          slot: slot.slot,
          name: sanitizeDisplayName(slot.name || `Guest ${slot.slot + 1}`),
          connected: slot.connected !== false,
          claim: slot.claim,
          life: slot.life,
          inCombat: !!slot.in_combat,
          hp: Number.isFinite(slot.hp) ? Math.max(0, Math.round(slot.hp)) : 0,
          maxHp: Number.isFinite(slot.max_hp) ? Math.max(0, Math.round(slot.max_hp)) : 0,
          block: Number.isFinite(slot.block) ? Math.max(0, Math.round(slot.block)) : 0,
        }));
    }
    return buildParticipantList(state, localName);
  }

  function currentSummary() {
    if (state.role === "guest") {
      return normalizeSummary(state.remoteFrame?.summary);
    }
    return normalizeSummary(state.localSummary);
  }

  function activeBarrierSlots(party) {
    if (!party || !Array.isArray(party.slots)) {
      return [];
    }
    return party.slots.filter(
      (slot) =>
        slot &&
        Number.isInteger(slot.slot) &&
        slot.slot < party.configured_party_size &&
        slot.in_run &&
        slot.connected !== false &&
        slot.life !== "dead",
    );
  }

  function currentBarrierWaitState(party) {
    if (!party || !PLAYER_BARRIER_SCREENS.has(party.screen)) {
      return null;
    }
    const slots = activeBarrierSlots(party);
    if (!slots.length) {
      return null;
    }
    const localSlot = slots.find((slot) => slot.slot === party.local_slot);
    if (!localSlot?.ready) {
      return null;
    }
    if (slots.every((slot) => !!slot.ready)) {
      return null;
    }
    return {
      title: "Waiting on players",
      subtitle: "Other players are still choosing.",
    };
  }

  function connectedPeers() {
    return Array.from(state.peers.values())
      .filter((peer) => peer.dc?.readyState === "open")
      .sort((left, right) => left.joinIndex - right.joinIndex);
  }

  function roomParticipants() {
    if (state.role === "host") {
      return [
        {
          peerId: state.localPeerId,
          name: localName(),
          connected: true,
          isLocal: true,
        },
        ...connectedPeers().map((peer) => ({
          peerId: peer.peerId || peer.fallbackPeerId,
          name: sanitizeDisplayName(peer.name || "Guest"),
          connected: true,
          isLocal: false,
        })),
      ];
    }
    if (state.role === "guest" && Array.isArray(state.guestConnection?.roomParticipants)) {
      return state.guestConnection.roomParticipants;
    }
    return [
      {
        peerId: state.localPeerId,
        name: localName(),
        connected: true,
        isLocal: true,
      },
    ];
  }

  function currentJoinedCount() {
    return roomParticipants().filter((participant) => participant.connected !== false).length;
  }

  function currentRoomHeading() {
    if (state.room.mode === "entry") {
      return {
        title: "Multiplayer",
        subtitle: "Peer-to-peer LAN. Choose whether this device hosts or joins.",
      };
    }
    if (state.room.mode === "claim-slot") {
      return {
        title: "Claim Slot",
        subtitle: "Pick a disconnected live hero to keep the run moving.",
      };
    }
    return {
      title: "Show QR to guests",
      subtitle: "",
    };
  }

  function replaceGuestRoomScreen(title, subtitle, bodyChildren = [], actionsChildren = []) {
    const shell = uiShell({
      centered: true,
      className: `multiplayer-room-shell multiplayer-${state.room.mode}-shell`,
    });
    const card = uiCard({ className: "multiplayer-modal-card multiplayer-guest-card" });
    const stack = uiStack();
    for (const child of bodyChildren) {
      if (child) {
        stack.appendChild(child);
      }
    }
    appendChildren(card, buildRoomCopy({ title, subtitle }), stack);
    if (actionsChildren.length > 0) {
      const actions = uiActions({ grow: true });
      for (const child of actionsChildren) {
        if (child) {
          actions.appendChild(child);
        }
      }
      card.appendChild(actions);
    }
    shell.appendChild(card);
    root.replaceChildren(shell);
  }

  function hostCanStart() {
    return state.role === "host" && !state.sessionStarted && currentJoinedCount() >= 2;
  }

  function canKeepInviting() {
    return state.role === "host" && !state.sessionStarted && currentJoinedCount() < MAX_PARTY_SIZE;
  }

  function partySizeNow() {
    const party = currentPartySnapshot();
    if (Number.isFinite(party?.configured_party_size)) {
      return clampPartySize(party.configured_party_size, 1);
    }
    return clampPartySize(state.partySize, 1);
  }

  function resetPairingState() {
    options.resetPairTransportAssembly?.();
    state.room.pairing = null;
    state.room.guestResumeAction = null;
    state.room.transportProgress = null;
    state.room.scannerDetection = null;
    state.room.localQrFrames = [];
  }

  function setGuestResumeAction(label, status, note = "") {
    state.room.guestResumeAction = {
      label,
      status,
      note,
    };
  }

  function pairingTransportMode() {
    return state.room.transportMode === "direct" ? "direct" : "qr_loop";
  }

  function buildAnimatedTransportFrames(payload) {
    const frames = options.buildPairTransportFrames?.(
      payload,
      PAIR_QR_CHUNK_PAYLOAD_CHARS,
    );
    if (Array.isArray(frames) && frames.length > 0) {
      return frames;
    }
    throw new Error("Pair transport encoder is unavailable.");
  }

  async function encodePairPayloadFromHost(payload) {
    const encoded = await options.encodePairPayload?.(payload);
    if (typeof encoded === "string" && encoded.length > 0) {
      return encoded;
    }
    throw new Error("Pair payload encoder is unavailable.");
  }

  async function consumePairTransportTextFromHost(rawCode) {
    const consumed = options.consumePairTransportText?.(rawCode);
    if (consumed && typeof consumed === "object") {
      return consumed;
    }
    throw new Error("Pair payload decoder is unavailable.");
  }

  function beginLocalPairTransport(payload, invitationId, { nextRoomMode = null } = {}) {
    const mode = pairingTransportMode();
    if (nextRoomMode) {
      state.room.mode = nextRoomMode;
    }
    state.room.pairing = {
      mode,
      invitationId,
    };
    state.room.localCode = payload;
    state.room.localQrFrames =
      mode === "direct" ? [payload] : buildAnimatedTransportFrames(payload);
    state.room.status = "";
    state.room.note = "";
    state.room.guestResumeAction = null;
  }

  async function resumeGuestScanMode() {
    state.room.mode = "guest-scan";
    state.room.localCode = "";
    state.room.localQrFrames = [];
    state.room.error = "";
    state.room.status = "Scan the host QR.";
    state.room.note = "";
    state.room.guestResumeAction = null;
    await renderRoom();
    if (state.room.inputMode === "camera") {
      await startScanner();
    }
  }

  function clearPendingConnection({ close = true } = {}) {
    const pending = state.room.pendingConnection;
    state.room.pendingConnection = null;
    state.room.invitationId = null;
    if (!pending) {
      return;
    }
    if (close && pending.closeOnCancel && pending.pc) {
      try {
        pending.pc.close();
      } catch {}
      if (pending.peer) {
        state.peers.delete(pending.peer.peerId || pending.peer.fallbackPeerId);
      }
    }
  }

  async function handleScannedCode(rawCode) {
    return applyRemoteCode(rawCode, { source: "scanner" });
  }

  function buildRoomCopy(heading, { status = "", note = "", error = "" } = {}) {
    const copy = uiHeaderCopy();
    appendChildren(
      copy,
      uiTitle(heading.title),
      (() => {
        const element = uiCopy(heading.subtitle, { tone: "muted" });
        if (element) {
          element.id = "multiplayer-room-subtitle";
        }
        return element;
      })(),
      (() => {
        const element = status ? uiCopy(status, { tone: "muted" }) : null;
        if (element) {
          element.id = "multiplayer-room-status";
        }
        return element;
      })(),
      (() => {
        const element = note ? uiCopy(note, { tone: "note" }) : null;
        if (element) {
          element.id = "multiplayer-room-note";
        }
        return element;
      })(),
      (() => {
        const element = error ? uiCopy(error, { tone: "error" }) : null;
        if (element) {
          element.id = "multiplayer-room-error";
        }
        return element;
      })(),
    );
    return copy;
  }

  function patchRoomTextNode(selector, text) {
    const element = root.querySelector(selector);
    if (!(element instanceof HTMLElement)) {
      return;
    }
    const hasText = typeof text === "string" && text.length > 0;
    element.textContent = hasText ? text : "";
    element.hidden = !hasText;
  }

  function patchRoomLiveSurface() {
    patchRoomTextNode("#multiplayer-room-status", state.room.status || "");
    patchRoomTextNode("#multiplayer-room-note", state.room.note || "");
    patchRoomTextNode("#multiplayer-room-error", state.room.error || "");

    const remoteCodeField = root.querySelector("#multiplayer-remote-code");
    if (remoteCodeField instanceof HTMLTextAreaElement) {
      remoteCodeField.value = state.room.manualInput || "";
    }

    const localCodeField = root.querySelector("#multiplayer-local-code");
    if (localCodeField instanceof HTMLTextAreaElement) {
      localCodeField.value = state.room.localCode || "";
    }

    const progress = state.room.transportProgress;
    const progressRoot = root.querySelector("#multiplayer-scanner-progress");
    const progressFill = root.querySelector("#multiplayer-scanner-progress-fill");
    const progressLabel = root.querySelector("#multiplayer-scanner-progress-label");
    const showProgress =
      progressRoot instanceof HTMLElement &&
      Number.isFinite(progress?.total) &&
      progress.total > 1 &&
      Number.isFinite(progress?.received) &&
      progress.received >= 0;
    if (progressRoot instanceof HTMLElement) {
      progressRoot.hidden = !showProgress;
    }
    if (showProgress) {
      const percent = Math.max(
        0,
        Math.min(100, Math.round((progress.received / progress.total) * 100)),
      );
      if (progressFill instanceof HTMLElement) {
        progressFill.style.width = `${percent}%`;
      }
      if (progressLabel instanceof HTMLElement) {
        progressLabel.textContent = `${progress.received}/${progress.total} QR frames`;
      }
    }

    const preview = root.querySelector("#multiplayer-scanner-preview");
    if (
      preview instanceof HTMLElement &&
      typeof state.room.scannerPreviewAspect === "string" &&
      state.room.scannerPreviewAspect.length > 0
    ) {
      preview.style.aspectRatio = state.room.scannerPreviewAspect;
    }

    const detectionBox = root.querySelector("#multiplayer-scanner-detection-box");
    if (detectionBox instanceof HTMLElement) {
      const detection = state.room.scannerDetection;
      const active =
        detection &&
        Number.isFinite(detection.leftPct) &&
        Number.isFinite(detection.topPct) &&
        Number.isFinite(detection.widthPct) &&
        Number.isFinite(detection.heightPct);
      detectionBox.hidden = !active;
      if (active) {
        detectionBox.style.left = `${detection.leftPct}%`;
        detectionBox.style.top = `${detection.topPct}%`;
        detectionBox.style.width = `${detection.widthPct}%`;
        detectionBox.style.height = `${detection.heightPct}%`;
        detectionBox.classList.toggle(
          "is-accepted",
          Number.isFinite(detection.acceptedAt) &&
            performance.now() - detection.acceptedAt <= 260,
        );
      } else {
        detectionBox.classList.remove("is-accepted");
      }
    }
  }

  function buildScannerPreview() {
    state.room.scannerVideoMountId += 1;
    const preview = createElement("div", {
      id: "multiplayer-scanner-preview",
      className: "multiplayer-scanner-preview",
    });
    if (typeof state.room.scannerPreviewAspect === "string" && state.room.scannerPreviewAspect.length > 0) {
      preview.style.aspectRatio = state.room.scannerPreviewAspect;
    }
    const video = createElement("video", {
      id: "multiplayer-scanner-video",
      className: "ui-video-surface multiplayer-scanner-video",
      attrs: {
        autoplay: true,
        muted: true,
        playsinline: true,
      },
      dataset: {
        mountId: state.room.scannerVideoMountId,
      },
    });
    const overlay = createElement("div", {
      className: "multiplayer-scanner-overlay",
    });
    overlay.appendChild(
      createElement("div", {
        id: "multiplayer-scanner-detection-box",
        className: "multiplayer-scanner-detection-box",
        attrs: {
          hidden: true,
        },
      }),
    );
    appendChildren(preview, video, overlay);
    return preview;
  }

  function buildScannerProgress() {
    const progress = createElement("div", {
      id: "multiplayer-scanner-progress",
      className: "multiplayer-scanner-progress",
      attrs: {
        hidden: true,
      },
    });
    const track = createElement("div", {
      className: "multiplayer-scanner-progress-track",
    });
    track.appendChild(
      createElement("div", {
        id: "multiplayer-scanner-progress-fill",
        className: "multiplayer-scanner-progress-fill",
      }),
    );
    appendChildren(
      progress,
      track,
      createElement("div", {
        id: "multiplayer-scanner-progress-label",
        className: "multiplayer-scanner-progress-label",
      }),
    );
    return progress;
  }

  function buildScannerControlRow() {
    const cameraRow = uiButtonRow({ grow: true });
    if (scannerSupported()) {
      cameraRow.appendChild(
        uiButton(state.room.scannerActive ? "Reset" : "Start camera", {
          action: state.room.scannerActive ? "reset-scanner" : "start-scanner",
          variant: "secondary",
        }),
      );
      cameraRow.appendChild(
        uiButton(scannerFacingToggleLabel(state.room.cameraFacing), {
          action: "toggle-scanner-facing",
          variant: "secondary",
        }),
      );
    } else {
      cameraRow.appendChild(uiCopy(cameraAvailabilityMessage(), { tone: "note", tag: "span" }));
    }
    return cameraRow;
  }

  function setScannerPreviewAspect(width, height) {
    if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) {
      return;
    }
    const aspectText = `${Math.round(width)} / ${Math.round(height)}`;
    if (state.room.scannerPreviewAspect === aspectText) {
      return;
    }
    state.room.scannerPreviewAspect = aspectText;
    const preview = root.querySelector("#multiplayer-scanner-preview");
    if (preview instanceof HTMLElement) {
      preview.style.aspectRatio = aspectText;
    }
  }

  function normalizeScannerDetection(bounds, sourceWidth, sourceHeight) {
    if (
      !bounds ||
      !Number.isFinite(sourceWidth) ||
      !Number.isFinite(sourceHeight) ||
      sourceWidth <= 0 ||
      sourceHeight <= 0
    ) {
      return null;
    }
    const left = Math.max(0, Math.min(sourceWidth, bounds.left));
    const top = Math.max(0, Math.min(sourceHeight, bounds.top));
    const right = Math.max(left, Math.min(sourceWidth, bounds.left + bounds.width));
    const bottom = Math.max(top, Math.min(sourceHeight, bounds.top + bounds.height));
    const width = right - left;
    const height = bottom - top;
    if (!(width > 0 && height > 0)) {
      return null;
    }
    return {
      leftPct: (left / sourceWidth) * 100,
      topPct: (top / sourceHeight) * 100,
      widthPct: (width / sourceWidth) * 100,
      heightPct: (height / sourceHeight) * 100,
    };
  }

  function updateScannerDetection(bounds, sourceWidth, sourceHeight, { accepted = false } = {}) {
    const normalized = normalizeScannerDetection(bounds, sourceWidth, sourceHeight);
    if (!normalized) {
      return;
    }
    const now = performance.now();
    const previous = state.room.scannerDetection;
    const next = {
      ...normalized,
      seenAt: now,
      acceptedAt: accepted
        ? now
        : Number.isFinite(previous?.acceptedAt)
          ? previous.acceptedAt
          : 0,
    };
    const changed =
      !previous ||
      Math.abs(previous.leftPct - next.leftPct) > 0.5 ||
      Math.abs(previous.topPct - next.topPct) > 0.5 ||
      Math.abs(previous.widthPct - next.widthPct) > 0.5 ||
      Math.abs(previous.heightPct - next.heightPct) > 0.5 ||
      (accepted && next.acceptedAt !== previous.acceptedAt);
    state.room.scannerDetection = next;
    if (changed) {
      patchRoomLiveSurface();
    }
  }

  function clearScannerDetection({ force = false } = {}) {
    if (!state.room.scannerDetection) {
      return;
    }
    if (!force) {
      const seenAt = Number.isFinite(state.room.scannerDetection.seenAt)
        ? state.room.scannerDetection.seenAt
        : 0;
      if (performance.now() - seenAt < 220) {
        return;
      }
    }
    state.room.scannerDetection = null;
    patchRoomLiveSurface();
  }

  function scannerBoundsFromBarcodeDetector(code) {
    const rect = code?.boundingBox;
    if (!rect) {
      return null;
    }
    return {
      left: Number(rect.x) || 0,
      top: Number(rect.y) || 0,
      width: Number(rect.width) || 0,
      height: Number(rect.height) || 0,
    };
  }

  function scannerBoundsFromJsQr(code) {
    const location = code?.location;
    if (!location) {
      return null;
    }
    const corners = [
      location.topLeftCorner,
      location.topRightCorner,
      location.bottomRightCorner,
      location.bottomLeftCorner,
    ].filter(Boolean);
    if (!corners.length) {
      return null;
    }
    const xs = corners.map((corner) => Number(corner.x) || 0);
    const ys = corners.map((corner) => Number(corner.y) || 0);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    return {
      left: minX,
      top: minY,
      width: maxX - minX,
      height: maxY - minY,
    };
  }

  function buildModeToggle(action, activeMode, modes) {
    const row = uiToggleRow();
    for (const mode of modes) {
      row.appendChild(
        uiButton(mode.label, {
          action,
          variant: activeMode === mode.value ? "primary" : "secondary",
          dataset: { mode: mode.value },
        }),
      );
    }
    return row;
  }

  async function renderRoom() {
    root.dataset.open = state.room.open ? "true" : "false";
    if (!state.room.open) {
      root.replaceChildren();
      return;
    }

    const heading = currentRoomHeading();
    if (state.room.mode === "entry") {
      const shell = uiShell({ centered: true });
      const card = uiCard({ className: "multiplayer-modal-card multiplayer-entry-card" });
      const actions = uiActions({ grow: true });
      appendChildren(
        actions,
        uiButton("Host", { action: "host-room", variant: "primary" }),
        uiButton("Join", { action: "guest-room", variant: "primary" }),
        uiButton("Back", { action: "leave-session", variant: "secondary" }),
      );
      appendChildren(
        card,
        buildRoomCopy(heading, { error: state.room.error }),
        actions,
      );
      shell.appendChild(card);
      root.replaceChildren(shell);
      return;
    }

    if (state.room.mode === "claim-slot") {
      const slotOptions = Array.isArray(state.guestConnection?.slotOptions)
        ? state.guestConnection.slotOptions
        : [];
      const shell = uiShell({ centered: true });
      const card = uiCard({ className: "multiplayer-modal-card multiplayer-claim-shell" });
      const grid = createElement("div", { className: "multiplayer-claim-grid" });
      if (slotOptions.length) {
        for (const option of slotOptions) {
          const button = uiButton("", {
            action: "claim-slot",
            variant: "primary",
            className: "multiplayer-claim-card",
            attrs: { "aria-label": option.label },
            dataset: { slot: option.slot },
            children: [
              createElement("span", {
                className: "multiplayer-claim-title",
                text: option.label,
              }),
              createElement("span", {
                className: "multiplayer-claim-detail",
                text: option.detail,
              }),
            ],
          });
          grid.appendChild(button);
        }
      } else {
        grid.appendChild(uiChip("No reclaimable live slots right now.", { tone: "note" }));
      }
      const actions = uiActions({ grow: true });
      actions.appendChild(uiButton("Leave", { action: "leave-session", variant: "danger" }));
      appendChildren(
        card,
        buildRoomCopy(heading, {
          status: state.room.status || "Choose a slot to continue.",
          note: state.room.note,
          error: state.room.error,
        }),
        grid,
        actions,
      );
      shell.appendChild(card);
      root.replaceChildren(shell);
      return;
    }

    if (state.room.mode === "host-choice") {
      const shell = uiShell({ centered: true });
      const card = uiCard({ className: "multiplayer-modal-card multiplayer-entry-card" });
      const actions = uiActions({ grow: true });
      appendChildren(
        actions,
        uiButton("Resume Saved Run", {
          action: "host-resume-run",
          variant: "primary",
        }),
        uiButton("Start New Run", {
          action: "host-new-run",
          variant: "primary",
        }),
        uiButton("Back", { action: "entry-back", variant: "secondary" }),
      );
      appendChildren(
        card,
        buildRoomCopy(
          {
            title: "Host Multiplayer",
            subtitle: "Choose which run to host.",
          },
          { error: state.room.error },
        ),
        actions,
      );
      shell.appendChild(card);
      root.replaceChildren(shell);
      return;
    }

    if (state.room.mode === "guest-scan") {
      const body = [
        buildModeToggle("set-input-mode", state.room.inputMode, [
          { value: "camera", label: "Camera" },
          { value: "paste", label: "Paste code" },
        ]),
      ];
      if (state.room.note) {
        body.push(uiCopy(state.room.note, { tone: "note" }));
      }
      if (state.room.inputMode === "camera") {
        body.push(buildScannerPreview(), buildScannerProgress());
        body.push(buildScannerControlRow());
        const receivedTextarea = createElement("textarea", {
          id: "multiplayer-remote-code",
          className: "ui-input ui-textarea",
          attrs: {
            readonly: true,
            placeholder: "Scanned host code appears here.",
          },
        });
        receivedTextarea.value = state.room.manualInput || "";
        body.push(receivedTextarea);
      } else {
        const textarea = createElement("textarea", {
          id: "multiplayer-remote-code",
          className: "ui-input ui-textarea",
          attrs: {
            placeholder: "Paste a host QR frame or full host code here.",
          },
        });
        textarea.value = state.room.manualInput || "";
        body.push(textarea);
        const pasteButtons = uiButtonRow({ grow: true });
        pasteButtons.appendChild(
          uiButton("Use code", {
            action: "apply-remote-code",
            variant: "primary",
          }),
        );
        body.push(pasteButtons);
      }
      if (state.room.error) {
        body.push(uiCopy(state.room.error, { tone: "error" }));
      }
      replaceGuestRoomScreen(
        "Join Multiplayer",
        state.room.status || "Scan the host QR.",
        body,
        [uiButton("Leave", { action: "leave-session", variant: "danger" })],
      );
      const video = root.querySelector("#multiplayer-scanner-video");
      if (video instanceof HTMLVideoElement) {
        if (state.room.scannerStream) {
          video.srcObject = state.room.scannerStream;
          void video.play().catch(() => {});
        } else {
          video.srcObject = null;
        }
      }
      return;
    }

    if (state.room.mode === "guest-confirm") {
      const body = [];
      if (state.room.note) {
        body.push(uiCopy(state.room.note, { tone: "note" }));
      }
      if (state.room.localCode) {
        body.push(
          createElement("canvas", {
            id: "multiplayer-qr-canvas",
            className: "ui-canvas-surface multiplayer-qr-surface",
            attrs: {
              width: PAIR_QR_CANVAS_SIZE,
              height: PAIR_QR_CANVAS_SIZE,
            },
          }),
        );
      }
      const textarea = createElement("textarea", {
        id: "multiplayer-local-code",
        className: "ui-input ui-textarea multiplayer-guest-code",
        attrs: {
          readonly: true,
          placeholder: "Your confirm code appears here after you scan the host offer.",
        },
      });
      textarea.value = state.room.localCode || "";
      body.push(textarea);
      if (state.room.error) {
        body.push(uiCopy(state.room.error, { tone: "error" }));
      }
      replaceGuestRoomScreen(
        "Show Confirm Code",
        state.room.status || "",
        body,
        [
          state.room.guestResumeAction
            ? uiButton(state.room.guestResumeAction.label, {
                action: "resume-guest-scan",
                variant: "primary",
              })
            : null,
          uiButton("Copy", {
            action: "copy-local-code",
            variant: "secondary",
            disabled: !state.room.localCode,
          }),
          uiButton("Leave", { action: "leave-session", variant: "danger" }),
        ],
      );
      if (state.room.localCode) {
        await renderQr(
          root.querySelector("#multiplayer-qr-canvas"),
          state.room.localQrFrames.length ? state.room.localQrFrames : state.room.localCode || "",
        );
      }
      return;
    }

    if (state.room.mode === "guest-waiting") {
      const body = [];
      if (state.room.note) {
        body.push(uiCopy(state.room.note, { tone: "note" }));
      }
      if (state.room.error) {
        body.push(uiCopy(state.room.error, { tone: "error" }));
      }
      replaceGuestRoomScreen(
        "Waiting on host",
        state.room.status || "Connected. Waiting on host to start.",
        body,
        [uiButton("Leave", { action: "leave-session", variant: "danger" })],
      );
      return;
    }

    const participants = roomParticipants();
    const connectionStatus =
      state.room.status ||
      (state.role === "host"
        ? canKeepInviting()
          ? ""
          : "Room full. Press Start to begin."
        : state.guestConnection?.dc?.readyState === "open"
          ? "Connected. Waiting on host to start."
          : "Scan the host QR.");
    const shell = uiShell({ centered: true, className: "multiplayer-room-shell" });
    const card = uiCard({ className: "multiplayer-modal-card multiplayer-room-card" });
    const screen = createElement("div", { className: "multiplayer-room-screen" });
    const stack = uiStack({ className: "multiplayer-room-stack" });

    const outputSection = createElement("div", { className: "multiplayer-room-block" });
    if (state.room.localCode) {
      outputSection.appendChild(
        createElement("canvas", {
          id: "multiplayer-qr-canvas",
          className: "ui-canvas-surface multiplayer-qr-surface",
          attrs: {
            width: PAIR_QR_CANVAS_SIZE,
            height: PAIR_QR_CANVAS_SIZE,
          },
        }),
      );
    } else {
      outputSection.appendChild(uiCopy("Preparing a host code...", { tone: "note" }));
    }
    const localCodeField = createElement("textarea", {
      id: "multiplayer-local-code",
      className: "ui-input ui-textarea multiplayer-host-code",
      attrs: {
        readonly: true,
        placeholder: "Your host code appears here.",
      },
    });
    localCodeField.value = state.room.localCode || "";
    outputSection.appendChild(localCodeField);
    const outputButtons = uiButtonRow({ grow: true });
    outputButtons.appendChild(
      uiButton("Copy", {
        action: "copy-local-code",
        variant: "secondary",
        disabled: !state.room.localCode,
      }),
    );
    if (state.role === "host" && !state.sessionStarted) {
      outputButtons.appendChild(
        uiButton("New code", {
          action: "refresh-host-invite",
          variant: "secondary",
        }),
      );
    }
    outputSection.appendChild(outputButtons);

    const inputSection = createElement("div", { className: "multiplayer-room-block" });
    inputSection.appendChild(
      buildModeToggle("set-input-mode", state.room.inputMode, [
        { value: "camera", label: "Camera" },
        { value: "paste", label: "Paste code" },
      ]),
    );
    if (state.room.inputMode === "camera") {
      inputSection.appendChild(buildScannerPreview());
      inputSection.appendChild(buildScannerProgress());
      inputSection.appendChild(buildScannerControlRow());
      const receivedTextarea = createElement("textarea", {
        id: "multiplayer-remote-code",
        className: "ui-input ui-textarea",
        attrs: {
          readonly: true,
          placeholder: "Scanned guest or host code appears here.",
        },
      });
      receivedTextarea.value = state.room.manualInput || "";
      inputSection.appendChild(receivedTextarea);
    } else {
      const textarea = createElement("textarea", {
        id: "multiplayer-remote-code",
        className: "ui-input ui-textarea",
        attrs: {
          placeholder: "Paste a host or guest QR frame, or a full pairing code here.",
        },
      });
      textarea.value = state.room.manualInput || "";
      inputSection.appendChild(textarea);
      const pasteButtons = uiButtonRow({ grow: true });
      pasteButtons.appendChild(
        uiButton("Use code", {
          action: "apply-remote-code",
          variant: "secondary",
        }),
      );
      inputSection.appendChild(pasteButtons);
    }

    const playersSection = createElement("div", { className: "multiplayer-room-block" });
    playersSection.appendChild(
      uiCopy("Players", {
        tone: "muted",
        className: "multiplayer-room-label",
      }),
    );
    const roster = createElement("div", { className: "multiplayer-roster" });
    for (const participant of participants) {
      const row = createElement("div", {
        className: "multiplayer-roster-row",
        dataset: {
          local: participant.isLocal ? "true" : null,
          offline: participant.connected === false ? "true" : null,
        },
      });
      appendChildren(
        row,
        createElement("span", {
          className: "multiplayer-roster-name",
          text: sanitizeDisplayName(participant.name || "Player"),
        }),
        createElement("span", {
          className: "multiplayer-roster-state",
          text: participant.connected === false ? "offline" : participant.isLocal ? "you" : "joined",
        }),
      );
      roster.appendChild(row);
    }
    playersSection.appendChild(roster);

    appendChildren(stack, outputSection, inputSection, playersSection);

    const actions = uiActions({ grow: true });
    if (state.role === "host") {
      actions.appendChild(
        uiButton("Start", {
          action: "start-host-run",
          variant: "primary",
          disabled: !hostCanStart(),
        }),
      );
    }
    actions.appendChild(uiButton("Leave", { action: "leave-session", variant: "danger" }));

    appendChildren(
      screen,
      buildRoomCopy(heading, {
        status: connectionStatus,
        note: state.room.note,
        error: state.room.error,
      }),
      stack,
      actions,
    );
    card.appendChild(screen);
    shell.appendChild(card);
    root.replaceChildren(shell);

    if (state.room.localCode) {
      await renderQr(
        root.querySelector("#multiplayer-qr-canvas"),
        state.room.localQrFrames.length ? state.room.localQrFrames : state.room.localCode || "",
      );
    }

    const video = root.querySelector("#multiplayer-scanner-video");
    if (video instanceof HTMLVideoElement) {
      if (state.room.scannerStream) {
        video.srcObject = state.room.scannerStream;
        void video.play().catch(() => {});
      } else {
        video.srcObject = null;
      }
    }
    patchRoomLiveSurface();
  }

  async function startScanner() {
    if (state.room.scannerActive || !state.room.open || state.room.inputMode !== "camera") {
      return;
    }

    try {
      options.resetPairTransportAssembly?.();
      let stream = null;
      let forceJsQr = false;
      const override = await options.requestScannerStream?.({
        role: state.role,
        roomMode: state.room.mode,
        facingMode: state.room.cameraFacing,
      });
      if (override?.stream) {
        stream = override.stream;
        forceJsQr = override.forceJsQr === true;
      } else {
        if (!hasCameraCaptureSupport()) {
          state.room.error = "Camera access is unavailable in this browser.";
          await renderRoom();
          return;
        }
        try {
          stream = await window.navigator.mediaDevices.getUserMedia(
            preferredVideoConstraints(state.room.cameraFacing, true),
          );
        } catch {
          try {
            stream = await window.navigator.mediaDevices.getUserMedia(
              preferredVideoConstraints(state.room.cameraFacing, false),
            );
          } catch {
            stream = await window.navigator.mediaDevices.getUserMedia({
              video: {
                width: { ideal: 1280 },
                height: { ideal: 720 },
              },
            });
          }
        }
      }
      state.room.scannerStream = stream;
      state.room.scannerActive = true;
      state.room.error = "";
      state.room.transportProgress = null;
      state.room.scannerDetection = null;
      const scannerToken = state.room.scannerToken + 1;
      state.room.scannerToken = scannerToken;
      await renderRoom();
      const initialVideo = root.querySelector("#multiplayer-scanner-video");
      if (!(initialVideo instanceof HTMLVideoElement)) {
        return;
      }
      initialVideo.srcObject = stream;
      await initialVideo.play().catch(() => {});
      const frameSource =
        override?.frameSourceEl instanceof HTMLCanvasElement
          ? override.frameSourceEl
          : null;
      const detector = !forceJsQr && hasNativeQrDetector()
        ? new window.BarcodeDetector({ formats: ["qr_code"] })
        : null;
      const jsQr = detector ? null : await ensureJsQrLoaded();
      const fallbackCanvas = detector ? null : document.createElement("canvas");
      const fallbackContext =
        fallbackCanvas instanceof HTMLCanvasElement ? fallbackCanvas.getContext("2d") : null;
      let lastAcceptedCode = "";

      const scheduleNextScanAfterAcceptedCode = (scanResult) => {
        if (
          state.room.open &&
          state.room.scannerActive &&
          state.room.inputMode === "camera" &&
          state.room.scannerToken === scannerToken
        ) {
          window.setTimeout(
            () => {
              if (
                state.room.open &&
                state.room.scannerActive &&
                state.room.inputMode === "camera" &&
                state.room.scannerToken === scannerToken
              ) {
                window.requestAnimationFrame(scanFrame);
              }
            },
            scanResult === "partial"
              ? SCANNER_PARTIAL_SUCCESS_COOLDOWN_MS
              : SCANNER_COMPLETE_SUCCESS_COOLDOWN_MS,
          );
        }
      };

      const acceptScannedCode = async (rawValue, bounds, sourceWidth, sourceHeight) => {
        const isNewCode = rawValue !== lastAcceptedCode;
        const scanResult = isNewCode ? await handleScannedCode(rawValue) : false;
        if (bounds) {
          updateScannerDetection(bounds, sourceWidth, sourceHeight, {
            accepted: isNewCode && !!scanResult,
          });
        }
        if (isNewCode && scanResult) {
          lastAcceptedCode = rawValue;
          scheduleNextScanAfterAcceptedCode(scanResult);
          return true;
        }
        return false;
      };

      const scanFrame = async () => {
        if (
          !state.room.open ||
          !state.room.scannerActive ||
          state.room.inputMode !== "camera" ||
          state.room.scannerToken !== scannerToken
        ) {
          return;
        }
        try {
          const activeVideo =
            frameSource instanceof HTMLCanvasElement
              ? null
              : root.querySelector("#multiplayer-scanner-video");
          if (!(frameSource instanceof HTMLCanvasElement)) {
            if (!(activeVideo instanceof HTMLVideoElement)) {
              window.requestAnimationFrame(scanFrame);
              return;
            }
            if (activeVideo.srcObject !== stream) {
              activeVideo.srcObject = stream;
              void activeVideo.play().catch(() => {});
            }
          }
          const activeFrameSource =
            frameSource instanceof HTMLCanvasElement ? frameSource : activeVideo;
          const sourceReady =
            activeFrameSource instanceof HTMLCanvasElement
              ? activeFrameSource.width > 0 && activeFrameSource.height > 0
              : activeFrameSource instanceof HTMLVideoElement &&
                activeFrameSource.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA;
          if (sourceReady) {
            const sourceWidth = Math.max(
              1,
              activeFrameSource instanceof HTMLCanvasElement
                ? activeFrameSource.width
                : activeFrameSource instanceof HTMLVideoElement
                  ? activeFrameSource.videoWidth || activeFrameSource.clientWidth || 640
                  : 640,
            );
            const sourceHeight = Math.max(
              1,
              activeFrameSource instanceof HTMLCanvasElement
                ? activeFrameSource.height
                : activeFrameSource instanceof HTMLVideoElement
                  ? activeFrameSource.videoHeight || activeFrameSource.clientHeight || 480
                  : 480,
            );
            setScannerPreviewAspect(sourceWidth, sourceHeight);
            const injectedCode =
              typeof override?.readFrameCode === "function"
                ? override.readFrameCode(sourceWidth, sourceHeight)
                : null;
            if (injectedCode && typeof injectedCode.data === "string") {
              if (
                await acceptScannedCode(
                  injectedCode.data,
                  scannerBoundsFromJsQr(injectedCode),
                  sourceWidth,
                  sourceHeight,
                )
              ) {
                return;
              }
            } else if (detector) {
              const codes = await detector.detect(activeFrameSource);
              if (codes.length > 0 && typeof codes[0].rawValue === "string") {
                const bounds = scannerBoundsFromBarcodeDetector(codes[0]);
                const rawValue = codes[0].rawValue;
                if (await acceptScannedCode(rawValue, bounds, sourceWidth, sourceHeight)) {
                  return;
                }
              } else {
                clearScannerDetection();
              }
            } else if (jsQr && fallbackCanvas && fallbackContext) {
              if (fallbackCanvas.width !== sourceWidth || fallbackCanvas.height !== sourceHeight) {
                fallbackCanvas.width = sourceWidth;
                fallbackCanvas.height = sourceHeight;
              }
              fallbackContext.drawImage(activeFrameSource, 0, 0, sourceWidth, sourceHeight);
              const imageData = fallbackContext.getImageData(0, 0, sourceWidth, sourceHeight);
              const code = jsQr(imageData.data, sourceWidth, sourceHeight, {
                inversionAttempts: "attemptBoth",
              });
              if (code && typeof code.data === "string") {
                const bounds = scannerBoundsFromJsQr(code);
                if (await acceptScannedCode(code.data, bounds, sourceWidth, sourceHeight)) {
                  return;
                }
              } else {
                clearScannerDetection();
              }
            }
          } else {
            clearScannerDetection();
          }
        } catch (error) {
          console.error(error);
        }
        window.requestAnimationFrame(scanFrame);
      };

      window.requestAnimationFrame(scanFrame);
    } catch (error) {
      console.error(error);
      if (state.room.scannerStream) {
        for (const track of state.room.scannerStream.getTracks()) {
          track.stop();
        }
      }
      state.room.error = "Could not open the camera. Check permissions or use paste mode instead.";
      state.room.scannerActive = false;
      state.room.scannerStream = null;
      await renderRoom();
    }
  }

  async function stopScanner({ render = true } = {}) {
    state.room.scannerActive = false;
    state.room.scannerToken += 1;
    options.resetPairTransportAssembly?.();
    state.room.transportProgress = null;
    state.room.scannerDetection = null;
    state.room.scannerPreviewAspect = null;
    if (state.room.scannerStream) {
      for (const track of state.room.scannerStream.getTracks()) {
        track.stop();
      }
    }
    state.room.scannerStream = null;
    if (render) {
      await renderRoom();
    }
  }

  async function resetScanner() {
    if (!state.room.open || state.room.inputMode !== "camera") {
      return;
    }
    await stopScanner({ render: false });
    await startScanner();
  }

  async function toggleScannerFacingMode() {
    state.room.cameraFacing =
      state.room.cameraFacing === "environment" ? "user" : "environment";
    state.room.status = scannerFacingStatusLabel(state.room.cameraFacing);
    state.room.error = "";
    if (state.room.scannerActive) {
      await stopScanner({ render: false });
      await startScanner();
      return;
    }
    await renderRoom();
  }

  function applyPeerDisconnect(peer, { clearSlot = false } = {}) {
    if (!peer || !Number.isInteger(peer.slot)) {
      return;
    }
    if (clearSlot) {
      options.clearHostPartySlot?.(peer.slot);
    } else {
      options.disconnectHostPartySlot?.(peer.slot);
    }
  }

  function claimableSlotOptions() {
    if (!state.sessionStarted) {
      return [];
    }
    const party = currentPartySnapshot();
    if (!party?.slots) {
      return [];
    }
    return party.slots
      .filter(
        (slot) =>
          slot &&
          Number.isInteger(slot.slot) &&
          slot.slot < party.configured_party_size &&
          slot.slot !== party.local_slot &&
          slot.claim === "remote" &&
          slot.connected === false &&
          slot.life !== "dead",
      )
      .map((slot) =>
        normalizeSlotOption({
          slot: slot.slot,
          kind: "reclaim",
          name: slot.name || `Guest ${slot.slot + 1}`,
          label: `Slot ${slot.slot + 1} • ${sanitizeDisplayName(slot.name || `Guest ${slot.slot + 1}`)}`,
          detail: "Reconnect to this live hero.",
        }),
      )
      .filter(Boolean);
  }

  function buildHostRoomStatePayload() {
    return {
      type: "room_state",
      participants: roomParticipants(),
      joinedCount: currentJoinedCount(),
      canStart: hostCanStart(),
      started: state.sessionStarted,
    };
  }

  function broadcastRoomState() {
    if (state.role !== "host" || state.sessionStarted) {
      return;
    }
    const payload = buildHostRoomStatePayload();
    for (const peer of connectedPeers()) {
      safeSend(peer.dc, payload);
    }
  }

  function syncHostPeerSlot(peer) {
    if (state.role !== "host" || !Number.isInteger(peer?.slot)) {
      return false;
    }
    return !!options.syncHostPartySlot?.(peer.slot, {
      name: peer.name,
      connected: true,
      ready: false,
      inRun: false,
      inCombat: false,
      alive: true,
      hp: 0,
      maxHp: 0,
      block: 0,
    });
  }

  function buildHostSlotOptions(peer) {
    if (state.role !== "host" || !state.sessionStarted) {
      return [];
    }
    const options = claimableSlotOptions();
    if (Number.isInteger(peer?.slot)) {
      return options.filter((option) => option.slot === peer.slot);
    }
    return options;
  }

  function updatePeerSlotAssignment(peer) {
    const slot = Number.isInteger(peer.requestedSlot)
      ? peer.requestedSlot
      : Number.isInteger(peer.slot)
        ? peer.slot
        : null;
    if (!Number.isInteger(slot)) {
      return null;
    }
    const validSlot = buildHostSlotOptions(peer).some((option) => option.slot === slot);
    if (!validSlot) {
      return null;
    }
    if (peer.slot !== slot) {
      peer.lastRunSnapshotSent = null;
      peer.lastRunSnapshotVersionSent = null;
    }
    peer.slot = slot;
    peer.requestedSlot = null;
    syncHostPeerSlot(peer);
    return slot;
  }

  function sendLatestRunSnapshotToPeer(peer, { type = "run_snapshot", force = false } = {}) {
    if (
      state.role !== "host" ||
      peer?.dc?.readyState !== "open" ||
      !Number.isInteger(peer.slot)
    ) {
      return false;
    }
    const snapshot = options.getLocalRunSnapshot?.();
    if (typeof snapshot !== "string" || !snapshot.length) {
      return false;
    }
    const snapshotVersion = options.getLocalRunSnapshotVersion?.();
    if (!force && type === "run_snapshot") {
      if (
        Number.isFinite(snapshotVersion) &&
        peer.lastRunSnapshotVersionSent === snapshotVersion &&
        peer.lastRunSnapshotSent === peer.slot
      ) {
        return false;
      }
      if (
        !Number.isFinite(snapshotVersion) &&
        peer.lastRunSnapshotSent === snapshot &&
        peer.lastRunSnapshotVersionSent === peer.slot
      ) {
        return false;
      }
    }
    peer.lastRunSnapshotSent = Number.isFinite(snapshotVersion) ? peer.slot : snapshot;
    peer.lastRunSnapshotVersionSent = Number.isFinite(snapshotVersion)
      ? snapshotVersion
      : peer.slot;
    return safeSend(peer.dc, {
      type,
      snapshot,
      slot: peer.slot,
      partySize: partySizeNow(),
    });
  }

  function sendPostStartJoinState(peer) {
    if (state.role !== "host" || peer?.dc?.readyState !== "open") {
      return;
    }
    const slotOptions = buildHostSlotOptions(peer);
    if (slotOptions.length) {
      safeSend(peer.dc, {
        type: "slot_options",
        slot: Number.isInteger(peer.slot) ? peer.slot : null,
        options: slotOptions,
        partySize: partySizeNow(),
      });
      return;
    }
    safeSend(peer.dc, {
      type: "session_locked",
      message: "This run already started. Only disconnected live heroes can be reclaimed.",
    });
  }

  function resyncHostPeerSlots() {
    if (state.role !== "host" || !state.sessionStarted) {
      return;
    }
    for (const peer of connectedPeers()) {
      if (Number.isInteger(peer.slot)) {
        sendLatestRunSnapshotToPeer(peer);
      } else {
        sendPostStartJoinState(peer);
      }
    }
  }

  function currentBlockingScreen() {
    if (state.sessionFailure === "host_disconnected") {
      return {
        title: "Host Disconnected",
        subtitle: "The host left the session.",
        action: "main_menu",
        actionLabel: "Main Menu",
      };
    }
    if (!state.sessionStarted) {
      return null;
    }
    const party = currentPartySnapshot();
    if (!party || !Number.isInteger(party.local_slot) || !Number.isInteger(party.captain_slot)) {
      return null;
    }
    const barrierWait = currentBarrierWaitState(party);
    if (barrierWait) {
      return barrierWait;
    }
    if (state.role !== "guest") {
      return null;
    }
    if (party.local_slot === party.captain_slot) {
      return null;
    }
    if (party.screen === "map") {
      return {
        presentation: "banner",
        title: "Waiting on host",
        subtitle: "The host is choosing the next room.",
      };
    }
    return null;
  }

  function removePeer(peerKey, { clearSlot = false } = {}) {
    const peer = state.peers.get(peerKey);
    if (!peer) {
      return;
    }
    applyPeerDisconnect(peer, { clearSlot });
    if (clearSlot) {
      peer.slot = null;
    }
    try {
      peer.dc?.close();
    } catch {}
    try {
      peer.pc?.close();
    } catch {}
    state.peers.delete(peerKey);
  }

  async function leaveSession() {
    const shouldResetBootPartyConfig = state.room.open && !state.sessionStarted;
    await stopScanner();
    clearPendingConnection({ close: true });
    resetPairingState();
    if (state.guestConnection) {
      try {
        state.guestConnection.dc?.close();
      } catch {}
      try {
        state.guestConnection.pc?.close();
      } catch {}
      state.guestConnection = null;
    }
    for (const peerKey of Array.from(state.peers.keys())) {
      removePeer(peerKey, { clearSlot: true });
    }
    state.role = "none";
    state.sessionFailure = null;
    state.sessionStarted = false;
    state.localParty = null;
    state.remoteFrame = null;
    state.lastGuestRunSnapshot = "";
    state.lastSceneSent = "";
    state.lastSceneSentAt = 0;
    state.room.open = false;
    state.room.mode = "entry";
    state.room.hostRunMode = "new";
    state.room.status = "";
    state.room.note = "";
    state.room.error = "";
    state.room.localCode = "";
    state.room.localQrFrames = [];
    state.room.manualInput = "";
    state.room.awaitingPeerOpen = false;
    if (shouldResetBootPartyConfig) {
      options.setConfiguredPartySize?.(1);
    }
    await renderRoom();
    options.requestRender?.();
  }

  async function continueActiveSessionIfConnected() {
    if (!state.sessionStarted) {
      return false;
    }

    if (state.role === "guest") {
      if (!state.guestConnection?.dc || state.guestConnection.dc.readyState !== "open") {
        return false;
      }
      const activeSlot = Number.isInteger(state.guestConnection.slot)
        ? state.guestConnection.slot
        : 0;
      if (options.isBootScreen?.()) {
        if (
          typeof state.lastGuestRunSnapshot !== "string" ||
          state.lastGuestRunSnapshot.length === 0
        ) {
          return false;
        }
        const restored = !!options.applyGuestRunSnapshot?.(state.lastGuestRunSnapshot, activeSlot);
        if (!restored) {
          return false;
        }
        state.guestConnection.snapshotMode = true;
      }
    } else if (state.role === "host") {
      if (connectedPeers().length === 0) {
        return false;
      }
      if (options.isBootScreen?.()) {
        const restored = options.restoreSavedRun?.() ?? false;
        if (!restored) {
          return false;
        }
      }
    } else {
      return false;
    }

    state.sessionFailure = null;
    state.room.open = false;
    state.room.status = "";
    state.room.note = "";
    state.room.error = "";
    state.room.localCode = "";
    state.room.localQrFrames = [];
    state.room.manualInput = "";
    await stopScanner();
    await renderRoom();
    options.requestRender?.();
    return true;
  }

  async function openClaimSlotRoom({ error = "" } = {}) {
    state.sessionFailure = null;
    state.room.open = true;
    state.room.mode = "claim-slot";
    state.room.status = "Choose the slot this device should control.";
    state.room.note = "Disconnected live heroes can be reclaimed by any player who rejoins.";
    state.room.error = error;
    await renderRoom();
  }

  async function prepareHostInvite() {
    if (state.role !== "host" || state.sessionStarted || !canKeepInviting()) {
      state.room.localCode = "";
      resetPairingState();
      await renderRoom();
      return;
    }

    clearPendingConnection({ close: true });
    resetPairingState();
    const pc = new RTCPeerConnection({ iceServers: [] });
    const dc = pc.createDataChannel("mazocarta");
    const fallbackPeerId = `guest-${randomHex(4)}`;
    const invitationId = randomHex(PAIR_TOKEN_BYTES).toUpperCase();
    const peer = {
      fallbackPeerId,
      peerId: null,
      name: "Guest",
      pc,
      dc,
      joinIndex: state.nextJoinIndex++,
      slot: null,
      requestedSlot: null,
      lastRunSnapshotSent: null,
      lastRunSnapshotVersionSent: null,
    };
    state.peers.set(fallbackPeerId, peer);
    attachPeerHandlers(peer);

    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    await waitForIceComplete(pc);
    const encodedOffer = await encodePairPayloadFromHost({
      kind: "mazocarta_offer",
      invitationId,
      description: {
        type: pc.localDescription?.type || "offer",
        sdp: pc.localDescription?.sdp || "",
      },
    });
    options.resetPairTransportAssembly?.();
    beginLocalPairTransport(encodedOffer, invitationId);
    state.room.invitationId = invitationId;
    state.room.pendingConnection = {
      pc,
      dc,
      peer,
      closeOnCancel: true,
    };
    state.room.error = "";
    state.room.awaitingPeerOpen = false;
    await renderRoom();
    if (state.room.inputMode === "camera") {
      await startScanner();
    }
  }

  async function openHostRoom({
    preferPaste = false,
    transportMode = null,
    inputMode = null,
    hostRunMode = "new",
  } = {}) {
    await stopScanner();
    clearPendingConnection({ close: true });
    resetPairingState();
    const desiredTransportMode =
      transportMode === "direct" || transportMode === "qr_loop"
        ? transportMode
        : preferPaste
          ? "direct"
          : "qr_loop";
    const desiredInputMode =
      inputMode === "paste" || inputMode === "camera"
        ? inputMode
        : preferPaste
          ? "paste"
          : "camera";
    state.role = "host";
    state.sessionFailure = null;
    state.sessionStarted = false;
    state.lastGuestRunSnapshot = "";
    state.room.open = true;
    state.room.mode = "host-room";
    state.room.hostRunMode = hostRunMode === "resume" ? "resume" : "new";
    state.room.outputMode = "qr";
    state.room.transportMode = desiredTransportMode;
    state.room.inputMode = desiredInputMode;
    state.room.localCode = "";
    state.room.localQrFrames = [];
    state.room.manualInput = "";
    state.room.status = "";
    state.room.note = "";
    state.room.error = "";
    state.room.awaitingPeerOpen = false;
    await renderRoom();
    await prepareHostInvite();
  }

  function attachGuestHandlers(connection) {
    connection.dc.addEventListener("open", async () => {
      safeSend(connection.dc, {
        type: "hello",
        peerId: state.localPeerId,
        name: localName(),
      });
      if (state.room.pendingConnection) {
        state.room.pendingConnection.closeOnCancel = false;
      }
      clearPendingConnection({ close: false });
      resetPairingState();
      state.role = "guest";
      state.room.mode = "guest-waiting";
      state.room.localCode = "";
      state.room.status = "Connected. Waiting on host to start.";
      state.room.note = "";
      state.room.error = "";
      state.room.awaitingPeerOpen = false;
      await stopScanner();
      state.room.open = true;
      await renderRoom();
      options.requestRender?.();
    });
    connection.dc.addEventListener("message", (event) => {
      try {
        void handleGuestMessage(JSON.parse(event.data));
      } catch (error) {
        console.error(error);
      }
    });
    connection.dc.addEventListener("close", () => {
      if (state.sessionStarted) {
        void handleHostDisconnect();
        return;
      }
      void leaveSession();
    });
    connection.dc.addEventListener("error", (error) => console.error(error));
    connection.pc.addEventListener("connectionstatechange", () => {
      if (["closed", "failed", "disconnected"].includes(connection.pc.connectionState)) {
        if (state.sessionStarted) {
          void handleHostDisconnect();
          return;
        }
        void leaveSession();
      }
    });
  }

  async function openGuestRoom({
    preferPaste = false,
    transportMode = null,
    inputMode = null,
  } = {}) {
    await stopScanner();
    clearPendingConnection({ close: true });
    resetPairingState();
    const desiredTransportMode =
      transportMode === "direct" || transportMode === "qr_loop"
        ? transportMode
        : preferPaste
          ? "direct"
          : "qr_loop";
    const desiredInputMode =
      inputMode === "paste" || inputMode === "camera"
        ? inputMode
        : preferPaste
          ? "paste"
          : "camera";
    state.role = "guest";
    state.sessionFailure = null;
    state.sessionStarted = false;
    state.lastGuestRunSnapshot = "";
    state.room.open = true;
    state.room.mode = "guest-scan";
    state.room.outputMode = "qr";
    state.room.transportMode = desiredTransportMode;
    state.room.inputMode = desiredInputMode;
    state.room.localCode = "";
    state.room.localQrFrames = [];
    state.room.manualInput = "";
    state.room.status = "Scan the host QR.";
    state.room.note = "";
    state.room.error = "";
    state.room.awaitingPeerOpen = false;
    await renderRoom();
    await startScanner();
  }

  async function applyHostAnswer(payload) {
    if (payload.kind !== "mazocarta_answer" || payload.invitationId !== state.room.invitationId) {
      throw new Error("This confirm code does not match the active host invite.");
    }
    const pc = state.room.pendingConnection?.pc;
    if (!pc) {
      throw new Error("No active host invite to complete.");
    }
    if (state.room.awaitingPeerOpen) {
      return;
    }
    state.room.awaitingPeerOpen = true;
    await stopScanner();
    await pc.setRemoteDescription(payload.description);
    state.room.status = "Connecting guest...";
    state.room.note = "";
    state.room.error = "";
    await renderRoom();
  }

  async function applyGuestOffer(payload) {
    if (payload.kind !== "mazocarta_offer") {
      throw new Error("Expected a host offer code.");
    }

    const pc = new RTCPeerConnection({ iceServers: [] });
    const connection = {
      pc,
      dc: null,
      slot: null,
      slotOptions: [],
      snapshotMode: false,
      roomParticipants: [],
    };
    pc.addEventListener("datachannel", (event) => {
      connection.dc = event.channel;
      attachGuestHandlers(connection);
    });
    await pc.setRemoteDescription(payload.description);
    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);
    await waitForIceComplete(pc);
    state.guestConnection = connection;
    state.sessionFailure = null;
    const encodedAnswer = await encodePairPayloadFromHost({
      kind: "mazocarta_answer",
      invitationId: payload.invitationId,
      description: {
        type: pc.localDescription?.type || "answer",
        sdp: pc.localDescription?.sdp || "",
      },
    });
    options.resetPairTransportAssembly?.();
    beginLocalPairTransport(encodedAnswer, payload.invitationId, {
      nextRoomMode: "guest-confirm",
    });
    state.room.error = "";
    state.room.pendingConnection = {
      pc,
      closeOnCancel: true,
    };
    await stopScanner();
    await renderRoom();
  }

  async function applyRemoteCode(rawCode, { source = "manual" } = {}) {
    try {
      const consumed = await consumePairTransportTextFromHost(rawCode);
      state.room.error = "";
      if (consumed.status === "partial") {
        state.room.transportProgress = {
          received: consumed.received,
          total: consumed.total,
        };
        state.room.status = `Scanned ${consumed.received}/${consumed.total} QR frames.`;
        state.room.note = "";
        patchRoomLiveSurface();
        return "partial";
      }
      state.room.transportProgress = null;
      const fullCode =
        typeof consumed.fullCode === "string" && consumed.fullCode.length > 0
          ? consumed.fullCode
          : rawCode;
      state.room.manualInput = fullCode;
      const payload = consumed.payload;
      if (!payload || typeof payload !== "object") {
        throw new Error("Invalid pairing code.");
      }
      if (state.role === "host" && state.room.awaitingPeerOpen) {
        return "complete";
      }
      if (state.role === "host") {
        if (payload.kind === "mazocarta_join") {
          state.room.remoteGuestName = sanitizeDisplayName(payload.name || "Guest");
          state.room.status = `${state.room.remoteGuestName} is ready. Have them scan your host code, then scan their confirm code.`;
          await renderRoom();
          return "complete";
        }
        await applyHostAnswer(payload);
        return "complete";
      }
      if (state.role === "guest") {
        await applyGuestOffer(payload);
        return "complete";
      }
    } catch (error) {
      if (source === "scanner") {
        state.room.error = "";
        patchRoomLiveSurface();
        return false;
      }
      console.error(error);
      state.room.awaitingPeerOpen = false;
      state.room.transportProgress = null;
      state.room.error = error instanceof Error ? error.message : "Invalid pairing code.";
      if (
        state.role === "host" &&
        !state.sessionStarted &&
        state.room.open &&
        state.room.inputMode === "camera"
      ) {
        await startScanner();
      }
      await renderRoom();
      return false;
    }
  }

  async function startHostRun() {
    if (!hostCanStart()) {
      return;
    }
    const peers = connectedPeers();
    const joinedCount = peers.length + 1;
    const targetPartySize =
      state.room.hostRunMode === "resume"
        ? clampPartySize(Math.max(partySizeNow(), joinedCount), 1)
        : clampPartySize(joinedCount, 1);
    state.partySize = targetPartySize;
    options.setConfiguredPartySize?.(targetPartySize);
    for (let slot = targetPartySize; slot < MAX_PARTY_SIZE; slot += 1) {
      options.clearHostPartySlot?.(slot);
    }
    peers.forEach((peer, index) => {
      peer.slot = index + 1;
      peer.requestedSlot = null;
      peer.lastRunSnapshotSent = null;
      peer.lastRunSnapshotVersionSent = null;
      syncHostPeerSlot(peer);
    });
    const started =
      state.room.hostRunMode === "resume"
        ? !!options.resumeHostRun?.()
        : !!options.startHostRun?.();
    if (!started) {
      state.room.error = "Could not start the run.";
      await renderRoom();
      return;
    }
    state.sessionStarted = true;
    state.sessionFailure = null;
    state.room.open = false;
    await stopScanner();
    clearPendingConnection({ close: true });
    resetPairingState();
    state.room.status = "";
    state.room.note = "";
    state.room.error = "";
    state.room.localCode = "";
    state.room.manualInput = "";
    state.room.awaitingPeerOpen = false;
    for (const peer of peers) {
      sendLatestRunSnapshotToPeer(peer, { type: "start_run", force: true });
    }
    await renderRoom();
    options.requestRender?.();
  }

  async function handleHostMessage(peer, payload) {
    if (!payload || typeof payload !== "object") {
      return;
    }

    if (payload.type === "hello") {
      peer.peerId =
        typeof payload.peerId === "string" && payload.peerId.length > 0
          ? payload.peerId
          : peer.fallbackPeerId;
      peer.name = sanitizeDisplayName(payload.name || "Guest");
      if (!state.peers.has(peer.peerId)) {
        state.peers.delete(peer.fallbackPeerId);
        state.peers.set(peer.peerId, peer);
      }
      peer.requestedSlot = null;
      peer.lastRunSnapshotSent = null;
      peer.lastRunSnapshotVersionSent = null;
      if (state.sessionStarted) {
        sendPostStartJoinState(peer);
      } else {
        broadcastRoomState();
      }
      await renderRoom();
      options.requestRender?.();
      return;
    }

    if (payload.type === "claim_slot") {
      peer.requestedSlot = Number.isInteger(payload.slot) ? payload.slot : null;
      const assignedSlot = updatePeerSlotAssignment(peer);
      if (!Number.isInteger(assignedSlot)) {
        safeSend(peer.dc, {
          type: "reject",
          message: "That slot is no longer claimable. Pick another slot.",
        });
        sendPostStartJoinState(peer);
      } else {
        safeSend(peer.dc, {
          type: "slot_assignment",
          slot: assignedSlot,
          partySize: partySizeNow(),
        });
        sendLatestRunSnapshotToPeer(peer, { force: true });
      }
      await renderRoom();
      options.requestRender?.();
      return;
    }

    if (payload.type === "input" && Number.isInteger(peer.slot)) {
      const applied = options.applyHostInput?.({
        ...payload,
        slot: peer.slot,
      });
      if (payload.kind === "combat_action" && applied === false) {
        const snapshot = options.getLocalRunSnapshot?.();
        if (typeof snapshot === "string" && snapshot.length) {
          safeSend(peer.dc, {
            type: "combat_action_rejected",
            snapshot,
            slot: peer.slot,
            partySize: partySizeNow(),
          });
        }
      }
    }
  }

  async function handleGuestMessage(payload) {
    if (!payload || typeof payload !== "object") {
      return;
    }

    if (payload.type === "scene" && typeof payload.scene === "string") {
      state.remoteFrame = {
        scene: payload.scene,
        width: Number.isFinite(payload.width) ? payload.width : 1280,
        height: Number.isFinite(payload.height) ? payload.height : 720,
        summary: normalizeSummary(payload.summary),
        participants: Array.isArray(payload.participants) ? payload.participants : [],
        party: normalizePartySnapshot(payload.party),
      };
      options.requestRender?.();
      return;
    }

    if (payload.type === "room_state") {
      if (!state.guestConnection) {
        return;
      }
      state.guestConnection.roomParticipants = Array.isArray(payload.participants)
        ? payload.participants.map((participant) => ({
            peerId: participant.peerId,
            name: sanitizeDisplayName(participant.name || "Player"),
            connected: participant.connected !== false,
            isLocal: participant.peerId === state.localPeerId,
          }))
        : [];
      if (state.room.mode === "guest-waiting") {
        state.room.status = "Connected. Waiting on host to start.";
      }
      await renderRoom();
      return;
    }

    if (payload.type === "start_run" && typeof payload.snapshot === "string") {
      if (!state.guestConnection) {
        return;
      }
      state.sessionStarted = true;
      state.sessionFailure = null;
      state.lastGuestRunSnapshot = payload.snapshot;
      if (Number.isInteger(payload.slot)) {
        state.guestConnection.slot = payload.slot;
      }
      if (Number.isFinite(payload.partySize)) {
        state.partySize = clampPartySize(payload.partySize, 1);
      }
      const activeSlot = Number.isInteger(state.guestConnection.slot) ? state.guestConnection.slot : 0;
      const restored = !!options.applyGuestRunSnapshot?.(payload.snapshot, activeSlot);
      state.guestConnection.snapshotMode = restored;
      state.sessionFailure = null;
      state.room.open = false;
      state.room.localCode = "";
      state.room.localQrFrames = [];
      await stopScanner();
      await renderRoom();
      options.requestRender?.();
      return;
    }

    if (payload.type === "slot_options") {
      if (!state.guestConnection) {
        return;
      }
      state.guestConnection.slot = Number.isInteger(payload.slot) ? payload.slot : null;
      state.guestConnection.slotOptions = Array.isArray(payload.options)
        ? payload.options.map(normalizeSlotOption).filter(Boolean)
        : [];
      if (Number.isFinite(payload.partySize)) {
        state.partySize = clampPartySize(payload.partySize, 1);
      }
      await openClaimSlotRoom();
      options.requestRender?.();
      return;
    }

    if (payload.type === "slot_assignment") {
      if (!state.guestConnection) {
        return;
      }
      state.guestConnection.slot = Number.isInteger(payload.slot) ? payload.slot : null;
      state.guestConnection.slotOptions = [];
      if (Number.isFinite(payload.partySize)) {
        state.partySize = clampPartySize(payload.partySize, 1);
      }
      state.room.status = Number.isInteger(payload.slot)
        ? `Claimed slot ${payload.slot + 1}.`
        : "Slot claimed.";
      await renderRoom();
      return;
    }

    if (payload.type === "run_snapshot" && typeof payload.snapshot === "string") {
      if (!state.guestConnection) {
        return;
      }
      state.lastGuestRunSnapshot = payload.snapshot;
      if (Number.isInteger(payload.slot)) {
        state.guestConnection.slot = payload.slot;
      }
      if (Number.isFinite(payload.partySize)) {
        state.partySize = clampPartySize(payload.partySize, 1);
      }
      const activeSlot = Number.isInteger(state.guestConnection.slot) ? state.guestConnection.slot : 0;
      const restored = !!options.applyGuestRunSnapshot?.(payload.snapshot, activeSlot);
      if (!restored) {
        state.sessionStarted = false;
        state.room.open = true;
        state.room.error = "Could not restore the host run snapshot.";
        await renderRoom();
        options.requestRender?.();
        return;
      }
      state.guestConnection.snapshotMode = restored;
      const party = extractPartySnapshotFromRunSnapshot(payload.snapshot, activeSlot);
      state.remoteFrame = {
        ...(state.remoteFrame || {
          scene: "",
          width: 1280,
          height: 720,
          summary: null,
          participants: [],
        }),
        party,
      };
      state.room.open = false;
      state.room.status = "";
      state.room.note = "";
      state.room.error = "";
      state.room.localCode = "";
      state.room.localQrFrames = [];
      state.room.manualInput = "";
      await stopScanner();
      await renderRoom();
      options.requestRender?.();
      return;
    }

    if (payload.type === "combat_action_rejected" && typeof payload.snapshot === "string") {
      if (!state.guestConnection) {
        return;
      }
      state.lastGuestRunSnapshot = payload.snapshot;
      if (Number.isInteger(payload.slot)) {
        state.guestConnection.slot = payload.slot;
      }
      if (Number.isFinite(payload.partySize)) {
        state.partySize = clampPartySize(payload.partySize, 1);
      }
      const activeSlot = Number.isInteger(state.guestConnection.slot)
        ? state.guestConnection.slot
        : 0;
      const restored =
        options.rejectGuestCombatAction?.(payload.snapshot, activeSlot) ??
        options.applyGuestRunSnapshot?.(payload.snapshot, activeSlot);
      if (!restored) {
        state.sessionStarted = false;
        state.room.open = true;
        state.room.error = "Could not restore the host run snapshot.";
        await renderRoom();
        options.requestRender?.();
        return;
      }
      state.guestConnection.snapshotMode = !!restored;
      const party = extractPartySnapshotFromRunSnapshot(payload.snapshot, activeSlot);
      state.remoteFrame = {
        ...(state.remoteFrame || {
          scene: "",
          width: 1280,
          height: 720,
          summary: null,
          participants: [],
        }),
        party,
      };
      options.requestRender?.();
      return;
    }

    if (payload.type === "session_locked") {
      state.room.open = true;
      state.room.mode = "guest-scan";
      state.room.error =
        typeof payload.message === "string" && payload.message.length > 0
          ? payload.message
          : "The run already started and there are no reclaimable slots.";
      await stopScanner();
      await renderRoom();
      options.requestRender?.();
      return;
    }

    if (payload.type === "reject") {
      await openClaimSlotRoom({
        error:
          typeof payload.message === "string" && payload.message.length > 0
            ? payload.message
            : "The host rejected that slot claim.",
      });
      options.requestRender?.();
    }
  }

  function attachPeerHandlers(peer) {
    peer.dc.addEventListener("open", async () => {
      if (state.room.pendingConnection?.peer === peer) {
        state.room.pendingConnection.closeOnCancel = false;
        clearPendingConnection({ close: false });
      }
      state.room.awaitingPeerOpen = false;
      state.room.status = "Guest connected. Add more players or press Start.";
      state.room.note = "";
      if (!state.sessionStarted) {
        await prepareHostInvite();
        broadcastRoomState();
      }
      await renderRoom();
      options.requestRender?.();
    });
    peer.dc.addEventListener("message", (event) => {
      try {
        const payload = JSON.parse(event.data);
        void handleHostMessage(peer, payload);
      } catch (error) {
        console.error(error);
      }
    });
    peer.dc.addEventListener("close", () => {
      state.room.awaitingPeerOpen = false;
      const peerKey = peer.peerId || peer.fallbackPeerId;
      applyPeerDisconnect(peer);
      state.peers.delete(peerKey);
      if (!state.sessionStarted) {
        broadcastRoomState();
        void prepareHostInvite();
      }
      void renderRoom();
      options.requestRender?.();
    });
    peer.dc.addEventListener("error", (error) => console.error(error));
    peer.pc.addEventListener("connectionstatechange", () => {
      if (["closed", "failed", "disconnected"].includes(peer.pc.connectionState)) {
        state.room.awaitingPeerOpen = false;
        const peerKey = peer.peerId || peer.fallbackPeerId;
        applyPeerDisconnect(peer);
        state.peers.delete(peerKey);
        if (!state.sessionStarted) {
          broadcastRoomState();
          void prepareHostInvite();
        }
        void renderRoom();
        options.requestRender?.();
      }
    });
  }

  function sendGuestInput(kind, payload = {}) {
    if (
      state.role !== "guest" ||
      !state.sessionStarted ||
      state.room.open ||
      currentBlockingScreen() ||
      !state.guestConnection?.dc ||
      state.guestConnection.dc.readyState !== "open"
    ) {
      return false;
    }
    safeSend(state.guestConnection.dc, {
      type: "input",
      kind,
      slot: Number.isInteger(state.guestConnection?.slot) ? state.guestConnection.slot : null,
      ...payload,
    });
    return true;
  }

  async function handleHostDisconnect() {
    if (state.role !== "guest" || !state.sessionStarted || state.sessionFailure === "host_disconnected") {
      return;
    }
    state.sessionFailure = "host_disconnected";
    state.room.open = false;
    state.room.status = "";
    state.room.note = "";
    state.room.error = "";
    state.room.localCode = "";
    state.room.localQrFrames = [];
    state.room.manualInput = "";
    await stopScanner();
    const connection = state.guestConnection;
    state.guestConnection = null;
    if (connection) {
      try {
        connection.dc?.close();
      } catch {}
      try {
        connection.pc?.close();
      } catch {}
    }
    await renderRoom();
    options.requestRender?.();
  }

  async function handleBlockingAction(action) {
    if (action !== "main_menu") {
      return false;
    }
    options.returnToMenu?.();
    options.setConfiguredPartySize?.(1);
    await leaveSession();
    options.requestRender?.();
    return true;
  }

  async function onUiClick(event) {
    const target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }
    const action = target.dataset.action;
    if (!action) {
      return;
    }

    event.preventDefault();
    if (action === "host-room") {
      if (options.hasSavedRun?.()) {
        state.room.mode = "host-choice";
        state.room.error = "";
        await renderRoom();
        return;
      }
      await openHostRoom();
      return;
    }
    if (action === "entry-back") {
      state.room.mode = "entry";
      state.room.error = "";
      await renderRoom();
      return;
    }
    if (action === "host-new-run") {
      await openHostRoom({ hostRunMode: "new" });
      return;
    }
    if (action === "host-resume-run") {
      if (!options.restoreSavedRun?.()) {
        state.room.error = "Could not resume the saved run.";
        await renderRoom();
        return;
      }
      await openHostRoom({ hostRunMode: "resume" });
      return;
    }
    if (action === "guest-room") {
      await openGuestRoom();
      return;
    }
    if (action === "leave-session") {
      if (
        state.room.mode === "entry" &&
        performance.now() - state.room.entryOpenedAt < ENTRY_BACK_GUARD_MS
      ) {
        return;
      }
      await leaveSession();
      return;
    }
    if (action === "blocking-main-menu") {
      await handleBlockingAction("main_menu");
      return;
    }
    if (action === "set-output-mode") {
      state.room.outputMode = target.dataset.mode === "code" ? "code" : "qr";
      await renderRoom();
      return;
    }
    if (action === "set-input-mode") {
      state.room.inputMode = target.dataset.mode === "paste" ? "paste" : "camera";
      if (state.room.inputMode === "camera") {
        await renderRoom();
        await startScanner();
      } else {
        await stopScanner();
      }
      return;
    }
    if (action === "copy-local-code") {
      const raw = state.room.localCode || "";
      if (raw && navigator.clipboard?.writeText) {
        try {
          await navigator.clipboard.writeText(raw);
        } catch (error) {
          console.error(error);
          state.room.error = "Could not copy the code.";
        }
        await renderRoom();
      }
      return;
    }
    if (action === "refresh-host-invite") {
      await prepareHostInvite();
      return;
    }
    if (action === "start-scanner") {
      await startScanner();
      return;
    }
    if (action === "reset-scanner") {
      await resetScanner();
      return;
    }
    if (action === "toggle-scanner") {
      if (state.room.scannerActive) {
        await resetScanner();
      } else {
        await startScanner();
      }
      return;
    }
    if (action === "toggle-scanner-facing") {
      await toggleScannerFacingMode();
      return;
    }
    if (action === "apply-remote-code") {
      await applyRemoteCode(state.room.manualInput || "");
      return;
    }
    if (action === "resume-guest-scan") {
      await resumeGuestScanMode();
      return;
    }
    if (action === "start-host-run") {
      if (!target.hasAttribute("disabled")) {
        await startHostRun();
      }
      return;
    }
    if (action === "claim-slot") {
      const requestedSlot = Number.parseInt(target.dataset.slot || "", 10);
      if (
        state.role === "guest" &&
        state.guestConnection?.dc &&
        state.guestConnection.dc.readyState === "open" &&
        Number.isInteger(requestedSlot)
      ) {
        state.room.status = `Claiming slot ${requestedSlot + 1}...`;
        state.room.error = "";
        await renderRoom();
        safeSend(state.guestConnection.dc, {
          type: "claim_slot",
          slot: requestedSlot,
        });
      }
    }
  }

  root.addEventListener("click", (event) => {
    void onUiClick(event);
  });

  root.addEventListener("input", (event) => {
    const target = event.target;
    if (target instanceof HTMLTextAreaElement && target.id === "multiplayer-remote-code") {
      state.room.manualInput = target.value;
    }
  });

  async function broadcastLocalScene(scene, viewport) {
    if (state.role !== "host" || !state.sessionStarted || state.peers.size === 0) {
      return;
    }
    void scene;
    void viewport;
  }

  function routeGuestPointer(kind, normalizedPoint) {
    const blocking = currentBlockingScreen();
    if (
      state.role !== "guest" ||
      !state.sessionStarted ||
      !normalizedPoint ||
      state.room.open ||
      options.isBootScreen?.() ||
      (blocking && blocking.presentation !== "banner")
    ) {
      return false;
    }
    const mappedPayload = options.mapGuestPointerInput?.(kind, normalizedPoint);
    if (mappedPayload && typeof mappedPayload === "object") {
      if (mappedPayload.kind === "local_pointer") {
        return false;
      }
      sendGuestInput(mappedPayload.kind || kind, mappedPayload);
      return true;
    }
    const party = currentPartySnapshot();
    if (blocking?.presentation === "banner" && party?.screen === "map") {
      return kind !== "pointer_move";
    }
    if (party?.screen === "combat") {
      return kind !== "pointer_move";
    }
    return sendGuestInput(kind, {
      xNorm: normalizedPoint.xNorm,
      yNorm: normalizedPoint.yNorm,
    });
  }

  void renderRoom();

  return {
    currentRemoteFrame() {
      return state.role === "guest" ? state.remoteFrame : null;
    },
    currentBlockingScreen() {
      return currentBlockingScreen();
    },
    isRoomOpen() {
      return state.room.open;
    },
    shouldBlockGameplayInput() {
      return state.room.open || !!currentBlockingScreen();
    },
    openEntryFlow: async () => {
      if (await continueActiveSessionIfConnected()) {
        return true;
      }
      state.role = "none";
      state.sessionFailure = null;
      state.room.open = true;
      state.room.mode = "entry";
      state.room.entryOpenedAt = performance.now();
      state.room.hostRunMode = "new";
      state.room.status = "";
      state.room.note = "";
      state.room.error = "";
      state.room.localCode = "";
      state.room.localQrFrames = [];
      state.room.manualInput = "";
      await renderRoom();
      return true;
    },
    activateBlockingAction(action) {
      void handleBlockingAction(action);
      return action === "main_menu";
    },
    async debugOpenHostRoom() {
      await openHostRoom({ transportMode: "direct", inputMode: "paste" });
      return true;
    },
    async debugOpenEntryFlow() {
      await this.openEntryFlow();
      return true;
    },
    async debugOpenHostRoomQr() {
      await openHostRoom({ transportMode: "qr_loop", inputMode: "paste" });
      return true;
    },
    async debugOpenGuestRoom() {
      await openGuestRoom({ transportMode: "direct", inputMode: "paste" });
      return true;
    },
    async debugOpenGuestRoomQr() {
      await openGuestRoom({ transportMode: "qr_loop", inputMode: "paste" });
      return true;
    },
    async debugApplyRemoteCode(raw) {
      await applyRemoteCode(String(raw ?? ""));
      return !state.room.error;
    },
    async debugStartHostRun() {
      await startHostRun();
      return state.sessionStarted;
    },
    debugGetRole() {
      return state.role;
    },
    debugGetCameraFacing() {
      return state.room.cameraFacing;
    },
    async debugSendGuestPayload(payload) {
      if (!payload || typeof payload !== "object" || typeof payload.kind !== "string") {
        return false;
      }
      const { kind, ...rest } = payload;
      return sendGuestInput(kind, rest);
    },
    debugGetLocalCode() {
      return state.room.localCode || "";
    },
    debugGetLocalQrFrames() {
      return Array.isArray(state.room.localQrFrames)
        ? [...state.room.localQrFrames]
        : [];
    },
    debugGetManualInput() {
      return state.room.manualInput || "";
    },
    debugGetRoomMode() {
      return state.room.mode;
    },
    debugGetRoomStatus() {
      return state.room.status || "";
    },
    debugGetRoomError() {
      return state.room.error || "";
    },
    debugGetTransportProgress() {
      return state.room.transportProgress
        ? { ...state.room.transportProgress }
        : null;
    },
    debugGetScannerDetection() {
      return state.room.scannerDetection
        ? { ...state.room.scannerDetection }
        : null;
    },
    debugIsScannerActive() {
      return state.room.scannerActive;
    },
    async debugStartRoomCameraScanner() {
      state.room.inputMode = "camera";
      await renderRoom();
      await startScanner();
      return state.room.scannerActive;
    },
    async debugStopRoomCameraScanner() {
      await stopScanner();
      return !state.room.scannerActive;
    },
    async debugResetRoomCameraScanner() {
      await resetScanner();
      return state.room.scannerActive;
    },
    debugIsSessionStarted() {
      return state.sessionStarted;
    },
    debugGetParticipants() {
      return roomParticipants();
    },
    guestRendersLocalState() {
      return state.role === "guest" && !!state.guestConnection?.snapshotMode;
    },
    setLocalSummary(summary) {
      state.localSummary = normalizeSummary(summary);
    },
    setLocalPartySnapshot(snapshot) {
      state.localParty = normalizePartySnapshot(snapshot);
      const party = currentPartySnapshot();
      if (Number.isFinite(party?.configured_party_size)) {
        state.partySize = clampPartySize(party.configured_party_size, 1);
      }
      if (state.role === "host") {
        if (state.sessionStarted) {
          resyncHostPeerSlots();
        } else {
          broadcastRoomState();
        }
      }
    },
    setConfiguredPartySize(size) {
      state.partySize = clampPartySize(size, 1);
    },
    notifyLocalRunSnapshotChanged() {
      if (state.role !== "host" || !state.sessionStarted) {
        return;
      }
      for (const peer of connectedPeers()) {
        sendLatestRunSnapshotToPeer(peer);
      }
    },
    afterLocalRender(scene, viewport) {
      void broadcastLocalScene(scene, viewport);
    },
    handleGuestPointerMove(normalizedPoint) {
      return routeGuestPointer("pointer_move", normalizedPoint);
    },
    handleGuestPointerDown(normalizedPoint) {
      return routeGuestPointer("pointer_down", normalizedPoint);
    },
    handleGuestPointerUp(normalizedPoint) {
      return routeGuestPointer("pointer_up", normalizedPoint);
    },
    handleGuestKeyDown(keyCode) {
      if (
        state.role !== "guest" ||
        !state.sessionStarted ||
        state.room.open ||
        currentBlockingScreen()
      ) {
        return false;
      }
      const mappedPayload = options.mapGuestKeyInput?.(keyCode);
      if (mappedPayload && typeof mappedPayload === "object") {
        sendGuestInput(mappedPayload.kind || "key_down", mappedPayload);
        return true;
      }
      const party = currentPartySnapshot();
      if (party?.screen === "combat") {
        return true;
      }
      return sendGuestInput("key_down", { keyCode });
    },
    async destroy() {
      await leaveSession();
      root.remove();
    },
  };
}
