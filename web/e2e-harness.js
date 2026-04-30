const E2E_FAKE_CAMERA_QR_SIZE = 448;
const E2E_FAKE_CAMERA_INTERVAL_MS = 420;
const E2E_FAKE_CAMERA_CAPTURE_FPS = 12;
const E2E_AUTOPLAY_ACTION_WAIT = 0;
const E2E_AUTOPLAY_ACTION_OPENING_INTRO_CONTINUE = 1;
const E2E_AUTOPLAY_ACTION_LEVEL_INTRO_CONTINUE = 2;
const E2E_AUTOPLAY_ACTION_MODULE_SELECT = 3;
const E2E_AUTOPLAY_ACTION_REWARD_SELECT = 4;
const E2E_AUTOPLAY_ACTION_REWARD_SKIP = 5;
const E2E_AUTOPLAY_ACTION_EVENT_CHOICE = 6;
const E2E_AUTOPLAY_ACTION_REST_HEAL = 7;
const E2E_AUTOPLAY_ACTION_REST_UPGRADE = 8;
const E2E_AUTOPLAY_ACTION_SHOP_BUY = 9;
const E2E_AUTOPLAY_ACTION_SHOP_LEAVE = 10;
const E2E_AUTOPLAY_ACTION_COMBAT = 11;

const E2E_FIXTURE_CODES = new Map([
  ["host-first-card", 1],
  ["multiplayer-map", 2],
  ["multiplayer-reward-barrier", 3],
]);

export function installE2EHarness(env) {
  let frameCounter = 0;
  let ready = false;
  let readyError = null;
  let readyResolve = null;
  let readyReject = null;
  let fakeCameraConfig = null;
  let restoreFakeJsQr = null;
  let fakeQrModulePromise = null;
  const readyPromise = new Promise((resolve, reject) => {
    readyResolve = resolve;
    readyReject = reject;
  });

  function wasm() {
    return env.getWasm?.() || null;
  }

  async function ensureFakeQrModule() {
    if (!fakeQrModulePromise) {
      fakeQrModulePromise = import("./qrcode.bundle.mjs");
    }
    return fakeQrModulePromise;
  }

  async function createFakeScannerStream() {
    const config = fakeCameraConfig;
    if (!config || !Array.isArray(config.frames) || config.frames.length === 0) {
      return null;
    }
    const frames = config.frames.filter(
      (frame) => typeof frame === "string" && frame.length > 0,
    );
    if (!frames.length) {
      return null;
    }
    const qrModule = await ensureFakeQrModule();
    const canvas = document.createElement("canvas");
    const size =
      Number.isFinite(config.size) && config.size > 0
        ? Math.round(config.size)
        : E2E_FAKE_CAMERA_QR_SIZE;
    const intervalMs =
      Number.isFinite(config.intervalMs) && config.intervalMs >= 80
        ? Math.round(config.intervalMs)
        : E2E_FAKE_CAMERA_INTERVAL_MS;
    canvas.width = size;
    canvas.height = size;
    if (typeof canvas.captureStream !== "function") {
      throw new Error("Fake camera captureStream is unavailable.");
    }
    const stream = canvas.captureStream(E2E_FAKE_CAMERA_CAPTURE_FPS);
    let stopped = false;
    let timerId = 0;
    let currentFrame = frames[0];
    let installMockJsQr = null;
    const restoreJsQr = () => {
      if (restoreFakeJsQr) {
        restoreFakeJsQr();
        restoreFakeJsQr = null;
      }
    };
    if (config.mockJsQr) {
      restoreJsQr();
      const originalJsQr = window.jsQR;
      installMockJsQr = () => {
        window.jsQR = (_data, width, height) => ({
          data: currentFrame,
          location: {
            topLeftCorner: { x: 0, y: 0 },
            topRightCorner: { x: width, y: 0 },
            bottomRightCorner: { x: width, y: height },
            bottomLeftCorner: { x: 0, y: height },
          },
        });
      };
      installMockJsQr();
      restoreFakeJsQr = () => {
        if (originalJsQr === undefined) {
          delete window.jsQR;
        } else {
          window.jsQR = originalJsQr;
        }
      };
    }
    for (const track of stream.getTracks()) {
      track.addEventListener("ended", () => {
        stopped = true;
        if (timerId) {
          window.clearTimeout(timerId);
          timerId = 0;
        }
        restoreJsQr();
      });
    }
    const renderFrame = async (index) => {
      if (stopped) {
        return;
      }
      currentFrame = frames[index];
      installMockJsQr?.();
      await qrModule.toCanvas(canvas, frames[index], {
        width: canvas.width,
        margin: 3,
        color: {
          dark: "#000000",
          light: "#ffffff",
        },
        errorCorrectionLevel: "L",
      });
      if (stopped) {
        return;
      }
      installMockJsQr?.();
      timerId = window.setTimeout(() => {
        void renderFrame((index + 1) % frames.length);
      }, intervalMs);
    };
    await renderFrame(0);
    return {
      stream,
      forceJsQr: config.forceJsQr !== false,
      frameSourceEl: config.exposeFrameSource === false ? null : canvas,
      readFrameCode: config.mockJsQr
        ? (width, height) => ({
            data: currentFrame,
            location: {
              topLeftCorner: { x: 0, y: 0 },
              topRightCorner: { x: width, y: 0 },
              bottomRightCorner: { x: width, y: height },
              bottomLeftCorner: { x: 0, y: height },
            },
          })
        : null,
    };
  }

  function markReady() {
    if (ready) {
      return;
    }
    ready = true;
    if (typeof readyResolve === "function") {
      readyResolve();
      readyResolve = null;
      readyReject = null;
    }
  }

  function failReady(error) {
    readyError = error instanceof Error ? error : new Error(String(error));
    if (typeof readyReject === "function") {
      readyReject(readyError);
      readyResolve = null;
      readyReject = null;
    }
  }

  function e2eCombatHintText() {
    const exports = wasm();
    if (!exports || typeof exports.app_debug_combat_hint_code !== "function") {
      return null;
    }
    switch (exports.app_debug_combat_hint_code()) {
      case 1:
        return "Waiting on players";
      case 2:
        return "Resolving enemy turn...";
      case 3:
        return "Resolving action...";
      case 4:
        return "Resolving encounter...";
      case 5:
        return "Tap enemy";
      case 6:
        return "Tap player";
      case 7:
        return "Tap card or end turn";
      case 8:
        return "Insufficient energy";
      default:
        return "Other";
    }
  }

  function e2eCombatHitCenters() {
    const exports = wasm();
    if (
      !exports ||
      typeof exports.app_combat_hand_len !== "function" ||
      typeof exports.app_combat_enemy_count !== "function" ||
      typeof exports.app_combat_hand_card_center_x !== "function" ||
      typeof exports.app_combat_hand_card_center_y !== "function" ||
      typeof exports.app_combat_enemy_center_x !== "function" ||
      typeof exports.app_combat_enemy_center_y !== "function" ||
      typeof exports.app_combat_player_center_x !== "function" ||
      typeof exports.app_combat_player_center_y !== "function" ||
      typeof exports.app_combat_end_turn_center_x !== "function" ||
      typeof exports.app_combat_end_turn_center_y !== "function"
    ) {
      return null;
    }

    const handLen = exports.app_combat_hand_len();
    const enemyCount = exports.app_combat_enemy_count();
    const logicalSize = env.getLogicalSize?.() || { width: 0, height: 0 };
    return {
      logicalWidth: logicalSize.width,
      logicalHeight: logicalSize.height,
      hand: Array.from({ length: handLen }, (_, index) => ({
        x: exports.app_combat_hand_card_center_x(index),
        y: exports.app_combat_hand_card_center_y(index),
      })),
      enemy: Array.from({ length: enemyCount }, (_, index) => ({
        x: exports.app_combat_enemy_center_x(index),
        y: exports.app_combat_enemy_center_y(index),
      })),
      player: {
        x: exports.app_combat_player_center_x(),
        y: exports.app_combat_player_center_y(),
      },
      endTurn: {
        x: exports.app_combat_end_turn_center_x(),
        y: exports.app_combat_end_turn_center_y(),
      },
    };
  }

  function loadFixture(name) {
    const exports = wasm();
    if (!exports || typeof exports.app_load_e2e_fixture !== "function") {
      return false;
    }
    const code = E2E_FIXTURE_CODES.get(name);
    if (!Number.isInteger(code)) {
      return false;
    }
    const loaded = !!exports.app_load_e2e_fixture(code);
    if (loaded) {
      env.resetSnapshotGenerations?.();
      env.syncRunSaveSnapshot?.();
      env.syncPartySnapshot?.();
      env.drawFrame?.();
    }
    return loaded;
  }

  async function maybeLoadFixture() {
    if (!env.fixtureName) {
      return false;
    }
    if (!loadFixture(env.fixtureName)) {
      throw new Error(`Unknown or failed E2E fixture: ${env.fixtureName}`);
    }
    return true;
  }

  function setNextRunSeed(seed) {
    const exports = wasm();
    if (!exports || typeof exports.app_debug_set_next_run_seed !== "function") {
      return false;
    }
    let value;
    try {
      value = BigInt(seed);
    } catch {
      return false;
    }
    if (value < 0n) {
      value = 0n;
    }
    const low = Number(value & 0xffff_ffffn);
    const high = Number((value >> 32n) & 0xffff_ffffn);
    exports.app_debug_set_next_run_seed(low >>> 0, high >>> 0);
    return true;
  }

  function readAutoplayAction() {
    const exports = wasm();
    if (
      !exports ||
      typeof exports.app_debug_autoplay_action_code !== "function" ||
      typeof exports.app_debug_autoplay_action_param_a !== "function" ||
      typeof exports.app_debug_autoplay_action_param_b !== "function"
    ) {
      return null;
    }
    return {
      code: exports.app_debug_autoplay_action_code() >>> 0,
      paramA: exports.app_debug_autoplay_action_param_a() >>> 0,
      paramB: exports.app_debug_autoplay_action_param_b() >>> 0,
    };
  }

  function autoplayGuestPayload(action) {
    if (!action || !Number.isInteger(action.code)) {
      return null;
    }
    switch (action.code) {
      case E2E_AUTOPLAY_ACTION_OPENING_INTRO_CONTINUE:
        return { kind: "opening_intro_action" };
      case E2E_AUTOPLAY_ACTION_LEVEL_INTRO_CONTINUE:
        return { kind: "level_intro_action" };
      case E2E_AUTOPLAY_ACTION_MODULE_SELECT:
        return { kind: "module_select", index: action.paramA };
      case E2E_AUTOPLAY_ACTION_REWARD_SELECT:
        return { kind: "reward_select", index: action.paramA };
      case E2E_AUTOPLAY_ACTION_REWARD_SKIP:
        return { kind: "reward_skip" };
      case E2E_AUTOPLAY_ACTION_EVENT_CHOICE:
        return { kind: "event_choice", index: action.paramA };
      case E2E_AUTOPLAY_ACTION_REST_HEAL:
        return { kind: "rest_heal" };
      case E2E_AUTOPLAY_ACTION_REST_UPGRADE:
        return { kind: "rest_upgrade", index: action.paramA };
      case E2E_AUTOPLAY_ACTION_SHOP_BUY:
        return { kind: "shop_buy", index: action.paramA };
      case E2E_AUTOPLAY_ACTION_SHOP_LEAVE:
        return { kind: "shop_leave" };
      case E2E_AUTOPLAY_ACTION_COMBAT:
        return { kind: "combat_action", actionCode: action.paramA >>> 0 };
      default:
        return null;
    }
  }

  function advanceAutoplayClock() {
    const exports = wasm();
    if (!exports || typeof exports.app_tick !== "function") {
      return false;
    }
    for (let index = 0; index < 16; index += 1) {
      exports.app_tick(64);
    }
    env.drawFrame?.();
    return true;
  }

  function clearPendingCombatAction() {
    const exports = wasm();
    if (
      !exports ||
      typeof exports.app_clear_local_multiplayer_pending_combat_action !== "function"
    ) {
      return false;
    }
    exports.app_clear_local_multiplayer_pending_combat_action();
    env.drawFrame?.();
    return true;
  }

  async function performAutoplayStep() {
    const action = readAutoplayAction();
    if (!action || action.code === E2E_AUTOPLAY_ACTION_WAIT) {
      return {
        acted: false,
        waiting: true,
        advanced: advanceAutoplayClock(),
        action,
      };
    }

    const multiplayer = env.multiplayer;
    const exports = wasm();
    const role = multiplayer.debugGetRole?.() || "none";
    if (role === "guest" && multiplayer.debugIsSessionStarted?.()) {
      if (
        action.code === E2E_AUTOPLAY_ACTION_COMBAT &&
        typeof exports?.app_apply_local_multiplayer_combat_action_code === "function"
      ) {
        exports.app_apply_local_multiplayer_combat_action_code(action.paramA >>> 0);
        env.drawFrame?.();
      }
      const payload = autoplayGuestPayload(action);
      if (!payload || typeof multiplayer.debugSendGuestPayload !== "function") {
        if (action.code === E2E_AUTOPLAY_ACTION_COMBAT) {
          clearPendingCombatAction();
        }
        return { acted: false, waiting: true, action };
      }
      const sent = await multiplayer.debugSendGuestPayload(payload);
      if (sent) {
        env.drawFrame?.();
      } else if (action.code === E2E_AUTOPLAY_ACTION_COMBAT) {
        clearPendingCombatAction();
      }
      return { acted: !!sent, waiting: !sent, action };
    }

    if (!exports || typeof exports.app_debug_apply_autoplay_action !== "function") {
      return { acted: false, waiting: true, action };
    }
    const applied =
      exports.app_debug_apply_autoplay_action(
        action.code >>> 0,
        action.paramA >>> 0,
        action.paramB >>> 0,
      ) !== 0;
    if (applied) {
      env.drawFrame?.();
      void env.flushHostEffects?.({ allowPrivilegedAction: false });
    }
    return { acted: applied, waiting: !applied, action };
  }

  window.__MAZOCARTA_E2E__ = {
    isReady() {
      return ready;
    },
    async waitReady() {
      if (ready) {
        return;
      }
      if (readyError) {
        throw readyError;
      }
      await readyPromise;
    },
    restoreSnapshot(raw, { multiplayerRestore = false } = {}) {
      return env.restoreRunSnapshotRaw?.(String(raw ?? ""), {
        useMultiplayerRestore: !!multiplayerRestore,
      });
    },
    loadFixture(name) {
      return loadFixture(name);
    },
    getRunSnapshotRaw() {
      return env.getRunSnapshotRaw?.() || null;
    },
    getPartySnapshot() {
      return env.getPartySnapshot?.() || null;
    },
    getLogicalSize() {
      return env.getLogicalSize?.() || { width: 0, height: 0 };
    },
    getRunSaveGeneration() {
      return env.getRunSaveGeneration?.() || 0;
    },
    storeRunSnapshot(raw) {
      return env.writeStoredRun?.(String(raw ?? ""));
    },
    getPartyScreen() {
      return env.getPartySnapshot?.()?.screen || null;
    },
    isBootScreen() {
      const exports = wasm();
      return !!(
        exports &&
        typeof exports.app_is_boot_screen === "function" &&
        exports.app_is_boot_screen()
      );
    },
    returnToMenu() {
      const exports = wasm();
      if (!exports || typeof exports.app_return_to_menu !== "function") {
        return false;
      }
      exports.app_return_to_menu();
      env.syncRunSaveSnapshot?.();
      env.syncPartySnapshot?.();
      env.drawFrame?.();
      return true;
    },
    getResultCode() {
      const exports = wasm();
      return typeof exports?.app_debug_result_code === "function"
        ? exports.app_debug_result_code() >>> 0
        : 0;
    },
    getBlockingScreen() {
      return env.multiplayer.currentBlockingScreen?.() || null;
    },
    getBlockingScreenActionRect() {
      return env.getBlockingScreenActionRect?.() || null;
    },
    getBlockingScreenBannerRect() {
      return env.getBlockingScreenBannerRect?.() || null;
    },
    async openHostRoom() {
      return env.multiplayer.debugOpenHostRoom?.();
    },
    async openMultiplayerEntry() {
      return env.multiplayer.debugOpenEntryFlow?.();
    },
    async openHostRoomQr() {
      return env.multiplayer.debugOpenHostRoomQr?.();
    },
    async openGuestRoom() {
      return env.multiplayer.debugOpenGuestRoom?.();
    },
    async openGuestRoomQr() {
      return env.multiplayer.debugOpenGuestRoomQr?.();
    },
    async applyPairCode(raw) {
      return env.multiplayer.debugApplyRemoteCode?.(raw);
    },
    async startHostRun() {
      return env.multiplayer.debugStartHostRun?.();
    },
    async startHostRunWithSeed(seed) {
      if (!setNextRunSeed(seed)) {
        return false;
      }
      return env.multiplayer.debugStartHostRun?.();
    },
    setNextRunSeed(seed) {
      return setNextRunSeed(seed);
    },
    getAutoplayAction() {
      return readAutoplayAction();
    },
    async autoplayStep() {
      return performAutoplayStep();
    },
    getRole() {
      return env.multiplayer.debugGetRole?.() || "none";
    },
    getPairCode() {
      return env.multiplayer.debugGetLocalCode?.() || "";
    },
    getPairQrFrames() {
      return env.multiplayer.debugGetLocalQrFrames?.() || [];
    },
    getPairManualInput() {
      return env.multiplayer.debugGetManualInput?.() || "";
    },
    getRoomMode() {
      return env.multiplayer.debugGetRoomMode?.() || "";
    },
    isMultiplayerRoomOpen() {
      return !!env.multiplayer.isRoomOpen?.();
    },
    getCameraFacing() {
      return env.multiplayer.debugGetCameraFacing?.() || "";
    },
    getRoomStatus() {
      return env.multiplayer.debugGetRoomStatus?.() || "";
    },
    getRoomError() {
      return env.multiplayer.debugGetRoomError?.() || "";
    },
    getTransportProgress() {
      return env.multiplayer.debugGetTransportProgress?.() || null;
    },
    getScannerDetection() {
      return env.multiplayer.debugGetScannerDetection?.() || null;
    },
    isScannerActive() {
      return !!env.multiplayer.debugIsScannerActive?.();
    },
    async startRoomCameraScanner() {
      return !!(await env.multiplayer.debugStartRoomCameraScanner?.());
    },
    async stopRoomCameraScanner() {
      return !!(await env.multiplayer.debugStopRoomCameraScanner?.());
    },
    async resetRoomCameraScanner() {
      return !!(await env.multiplayer.debugResetRoomCameraScanner?.());
    },
    async installFakeQrCamera(frames, options = {}) {
      fakeCameraConfig = {
        frames: Array.isArray(frames) ? [...frames] : [],
        intervalMs:
          Number.isFinite(options.intervalMs) && options.intervalMs > 0
            ? options.intervalMs
            : E2E_FAKE_CAMERA_INTERVAL_MS,
        size:
          Number.isFinite(options.size) && options.size > 0
            ? options.size
            : E2E_FAKE_CAMERA_QR_SIZE,
        forceJsQr: options.forceJsQr !== false,
        exposeFrameSource: options.exposeFrameSource !== false,
        mockJsQr: options.mockJsQr === true,
      };
      return true;
    },
    clearFakeQrCamera() {
      fakeCameraConfig = null;
      if (restoreFakeJsQr) {
        restoreFakeJsQr();
        restoreFakeJsQr = null;
      }
      return true;
    },
    isSessionStarted() {
      return !!env.multiplayer.debugIsSessionStarted?.();
    },
    getParticipants() {
      return env.multiplayer.debugGetParticipants?.() || [];
    },
    getFrameCounter() {
      return frameCounter;
    },
    getHintText() {
      return e2eCombatHintText();
    },
    getCombatLockState() {
      const exports = wasm();
      return !!(
        exports &&
        typeof exports.app_debug_combat_input_locked === "function" &&
        exports.app_debug_combat_input_locked()
      );
    },
    getCombatLockMask() {
      const exports = wasm();
      return exports && typeof exports.app_debug_combat_lock_mask === "function"
        ? exports.app_debug_combat_lock_mask() >>> 0
        : 0;
    },
    getPendingCombatActionCode() {
      const exports = wasm();
      return exports &&
        typeof exports.app_debug_pending_local_multiplayer_combat_action_code === "function"
        ? exports.app_debug_pending_local_multiplayer_combat_action_code() >>> 0
        : 0;
    },
    getCombatPlaybackQueueLen() {
      const exports = wasm();
      return exports && typeof exports.app_debug_combat_playback_queue_len === "function"
        ? exports.app_debug_combat_playback_queue_len() >>> 0
        : 0;
    },
    getCombatActiveStatCount() {
      const exports = wasm();
      return exports && typeof exports.app_debug_combat_active_stat_count === "function"
        ? exports.app_debug_combat_active_stat_count() >>> 0
        : 0;
    },
    getCombatHitCenters() {
      return e2eCombatHitCenters();
    },
  };

  return {
    createFakeScannerStream,
    maybeLoadFixture,
    markReady,
    failReady,
    isReady() {
      return ready;
    },
    async waitReady() {
      if (ready) {
        return;
      }
      if (readyError) {
        throw readyError;
      }
      await readyPromise;
    },
    incrementFrameCounter() {
      frameCounter += 1;
    },
  };
}
