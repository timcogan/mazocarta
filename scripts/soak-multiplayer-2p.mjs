#!/usr/bin/env node

import { spawn } from "node:child_process";
import http from "node:http";
import process from "node:process";
import { chromium } from "@playwright/test";

const DEFAULT_BASE_URL = "http://127.0.0.1:4173";
const DEFAULT_RUNS = 5;
const DEFAULT_SEED_START = 1;
const DEFAULT_RUN_TIMEOUT_MS = 90_000;
const IDLE_LIMIT = 240;
const STEP_DELAY_MS = 1;

function parseArgs(argv) {
  let runs = DEFAULT_RUNS;
  let seedStart = DEFAULT_SEED_START;
  let timeoutMs = DEFAULT_RUN_TIMEOUT_MS;
  let failOnTimeout = false;
  for (let index = 0; index < argv.length; index += 1) {
    const flag = argv[index];
    if (flag === "--runs") {
      index += 1;
      runs = Math.max(1, Number.parseInt(argv[index] || "", 10) || DEFAULT_RUNS);
      continue;
    }
    if (flag === "--seed-start") {
      index += 1;
      seedStart = Math.max(1, Number.parseInt(argv[index] || "", 10) || DEFAULT_SEED_START);
      continue;
    }
    if (flag === "--timeout-ms") {
      index += 1;
      timeoutMs = Math.max(1, Number.parseInt(argv[index] || "", 10) || DEFAULT_RUN_TIMEOUT_MS);
      continue;
    }
    if (flag === "--fail-on-timeout") {
      failOnTimeout = true;
      continue;
    }
    if (flag === "--help" || flag === "-h") {
      printUsage(0);
    }
    printUsage(2, `Unknown flag: ${flag}`);
  }
  return { runs, seedStart, timeoutMs, failOnTimeout };
}

function printUsage(code, message = "") {
  if (message) {
    console.error(message);
  }
  console.error("Usage: npm run soak:2p -- --runs N --seed-start N --timeout-ms N [--fail-on-timeout]");
  process.exit(code);
}

function request(url) {
  return new Promise((resolve, reject) => {
    let settled = false;
    let req;
    const settle = (callback, value) => {
      if (settled) {
        return;
      }
      settled = true;
      req.removeListener("response", onResponse);
      req.removeListener("error", onError);
      req.removeListener("timeout", onTimeout);
      callback(value);
    };
    const onError = (error) => settle(reject, error);
    const onTimeout = () => {
      settle(reject, new Error(`Request timed out after 5000ms: ${url}`));
      req.destroy();
    };
    const onResponse = (res) => {
      res.resume();
      settle(resolve, res.statusCode || 0);
    };
    req = http.get(url);
    req.setTimeout(5000, onTimeout);
    req.on("response", onResponse);
    req.on("error", onError);
  });
}

async function ensureWebServer(baseUrl) {
  try {
    const status = await request(baseUrl);
    if (status > 0) {
      return { child: null };
    }
  } catch {}

  const child = spawn("python3", ["-m", "http.server", "4173", "--directory", "web"], {
    stdio: "ignore",
  });
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    try {
      const status = await request(baseUrl);
      if (status > 0) {
        return { child };
      }
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  child.kill("SIGTERM");
  throw new Error("Timed out waiting for local web server.");
}

async function openE2EPage(page, baseUrl) {
  await page.goto(`${baseUrl}/?e2e=1`);
  await page.evaluate(async () => {
    await window.__MAZOCARTA_E2E__.waitReady();
  });
}

async function pairHostGuest(browser, baseUrl) {
  const hostContext = await browser.newContext();
  const guestContext = await browser.newContext();
  const hostPage = await hostContext.newPage();
  const guestPage = await guestContext.newPage();
  await Promise.all([openE2EPage(hostPage, baseUrl), openE2EPage(guestPage, baseUrl)]);

  await hostPage.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoom());
  await guestPage.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoom());

  const waitForCode = async (page) => {
    const deadline = Date.now() + 10_000;
    while (Date.now() < deadline) {
      const code = await page.evaluate(() => window.__MAZOCARTA_E2E__.getPairCode());
      if (typeof code === "string" && code.length > 0) {
        return code;
      }
      await page.waitForTimeout(50);
    }
    throw new Error("Timed out waiting for pairing code.");
  };

  const hostOffer = await waitForCode(hostPage);
  await guestPage.evaluate((code) => window.__MAZOCARTA_E2E__.applyPairCode(code), hostOffer);

  const guestAnswer = await waitForCode(guestPage);
  await hostPage.evaluate((code) => window.__MAZOCARTA_E2E__.applyPairCode(code), guestAnswer);

  const waitForSessionStarted = async (page) => {
    const deadline = Date.now() + 15_000;
    while (Date.now() < deadline) {
      const started = await page.evaluate(() => window.__MAZOCARTA_E2E__.isSessionStarted());
      if (started) {
        return;
      }
      await page.waitForTimeout(50);
    }
    throw new Error("Timed out waiting for multiplayer session start.");
  };

  return {
    hostContext,
    guestContext,
    hostPage,
    guestPage,
    async start(seed) {
      const started = await hostPage.evaluate(
        (value) => window.__MAZOCARTA_E2E__.startHostRunWithSeed(value),
        seed,
      );
      if (!started) {
        throw new Error(`Could not start host run for seed ${seed}.`);
      }
      await Promise.all([waitForSessionStarted(hostPage), waitForSessionStarted(guestPage)]);
    },
    async close() {
      await Promise.allSettled([hostContext.close(), guestContext.close()]);
    },
  };
}

async function readRunState(page) {
  return page.evaluate(() => ({
    role: window.__MAZOCARTA_E2E__.getRole(),
    partyScreen: window.__MAZOCARTA_E2E__.getPartyScreen(),
    party: window.__MAZOCARTA_E2E__.getPartySnapshot(),
    resultCode: window.__MAZOCARTA_E2E__.getResultCode(),
    blocking: window.__MAZOCARTA_E2E__.getBlockingScreen(),
    generation: window.__MAZOCARTA_E2E__.getRunSaveGeneration(),
    action: window.__MAZOCARTA_E2E__.getAutoplayAction?.() || null,
    locked: !!window.__MAZOCARTA_E2E__.getCombatLockState?.(),
    lockMask: window.__MAZOCARTA_E2E__.getCombatLockMask?.() || 0,
    pendingActionCode: window.__MAZOCARTA_E2E__.getPendingCombatActionCode?.() || 0,
    playbackQueueLen: window.__MAZOCARTA_E2E__.getCombatPlaybackQueueLen?.() || 0,
    activeStatCount: window.__MAZOCARTA_E2E__.getCombatActiveStatCount?.() || 0,
    hintText: window.__MAZOCARTA_E2E__.getHintText?.() || "",
    roomError: window.__MAZOCARTA_E2E__.getRoomError(),
  }));
}

async function autoplayBoth(hostPage, guestPage) {
  const [hostStep, guestStep] = await Promise.all([
    hostPage.evaluate(() => window.__MAZOCARTA_E2E__.autoplayStep()),
    guestPage.evaluate(() => window.__MAZOCARTA_E2E__.autoplayStep()),
  ]);
  return { hostStep, guestStep };
}

async function runSeed(browser, baseUrl, seed, { timeoutMs }) {
  const session = await pairHostGuest(browser, baseUrl);
  try {
    await session.start(seed);
    let idleTicks = 0;
    let iterations = 0;
    const startedAt = Date.now();

    while (Date.now() - startedAt < timeoutMs) {
      const [hostState, guestState] = await Promise.all([
        readRunState(session.hostPage),
        readRunState(session.guestPage),
      ]);

      if (hostState.resultCode === 1 && guestState.resultCode === 1) {
        return { outcome: "victory", seed, iterations, hostState, guestState };
      }
      if (hostState.resultCode === 2 || guestState.resultCode === 2) {
        return { outcome: "defeat", seed, iterations, hostState, guestState };
      }

      const { hostStep, guestStep } = await autoplayBoth(session.hostPage, session.guestPage);
      iterations += 1;
      if (hostStep?.acted || guestStep?.acted) {
        idleTicks = 0;
      } else {
        idleTicks += 1;
      }
      if (idleTicks >= IDLE_LIMIT) {
        return {
          outcome: "stall",
          seed,
          iterations,
          hostState,
          guestState,
          hostStep,
          guestStep,
        };
      }
      await session.hostPage.waitForTimeout(STEP_DELAY_MS);
    }

    const [hostState, guestState] = await Promise.all([
      readRunState(session.hostPage),
      readRunState(session.guestPage),
    ]);
    return { outcome: "progress_timeout", seed, iterations, hostState, guestState };
  } finally {
    await session.close();
  }
}

function formatBlocking(blocking) {
  if (!blocking || typeof blocking !== "object") {
    return "none";
  }
  return [blocking.title, blocking.subtitle].filter(Boolean).join(" / ") || "present";
}

function formatAction(action) {
  if (!action || typeof action !== "object") {
    return "none";
  }
  return `${action.code}:${action.paramA}:${action.paramB}`;
}

function formatReadySlots(party) {
  if (!party || !Array.isArray(party.slots)) {
    return "none";
  }
  return party.slots
    .filter((slot) => slot && Number.isInteger(slot.slot) && slot.slot < party.configured_party_size)
    .map((slot) => `${slot.slot}:${slot.ready ? "ready" : "open"}:${slot.life || "unknown"}`)
    .join(",");
}

function formatLockMask(mask) {
  const flags = [
    [1 << 0, "local-ready"],
    [1 << 1, "pending-local"],
    [1 << 2, "auto-playback"],
    [1 << 3, "pending-outcome"],
    [1 << 4, "pause"],
    [1 << 5, "active-stats"],
    [1 << 6, "stat-queue"],
  ];
  const labels = flags.filter(([bit]) => (mask & bit) !== 0).map(([, label]) => label);
  return labels.length ? labels.join("+") : "none";
}

function formatRunState(state) {
  return [
    `${state.partyScreen}/${formatBlocking(state.blocking)}`,
    `locked=${state.locked ? "yes" : "no"}`,
    `lock=${formatLockMask(state.lockMask)}`,
    `hint=${JSON.stringify(state.hintText)}`,
    `action=${formatAction(state.action)}`,
    `pending=${state.pendingActionCode}`,
    `pbq=${state.playbackQueueLen}`,
    `stats=${state.activeStatCount}`,
    `ready=${formatReadySlots(state.party)}`,
  ].join(" ");
}

async function main() {
  const { runs, seedStart, timeoutMs, failOnTimeout } = parseArgs(process.argv.slice(2));
  const { child: serverChild } = await ensureWebServer(DEFAULT_BASE_URL);
  const browser = await chromium.launch({ headless: true });
  const summary = {
    runs,
    victories: 0,
    defeats: 0,
    stalls: 0,
    progressTimeouts: 0,
    firstStallingSeed: null,
  };

  try {
    for (let offset = 0; offset < runs; offset += 1) {
      const seed = seedStart + offset;
      const result = await runSeed(browser, DEFAULT_BASE_URL, seed, { timeoutMs });
      if (result.outcome === "victory") {
        summary.victories += 1;
      } else if (result.outcome === "defeat") {
        summary.defeats += 1;
      } else if (result.outcome === "stall") {
        summary.stalls += 1;
      } else {
        summary.progressTimeouts += 1;
      }

      if (result.outcome === "stall" && summary.firstStallingSeed === null) {
        summary.firstStallingSeed = seed;
      }

      console.log(
        `seed=${seed} outcome=${result.outcome} iterations=${result.iterations} host=${formatRunState(result.hostState)} guest=${formatRunState(result.guestState)}`,
      );
    }
  } finally {
    await browser.close();
    if (serverChild) {
      serverChild.kill("SIGTERM");
    }
  }

  console.log(
    `runs=${summary.runs} victories=${summary.victories} defeats=${summary.defeats} stalls=${summary.stalls} progress_timeouts=${summary.progressTimeouts} first_stalling_seed=${summary.firstStallingSeed ?? "none"}`,
  );
  process.exit(summary.stalls > 0 || (failOnTimeout && summary.progressTimeouts > 0) ? 1 : 0);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
