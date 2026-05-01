import { expect, test } from "@playwright/test";

async function openHostFirstCardFixture(page) {
  await page.goto("/?e2e=1&fixture=host-first-card");
  await page.evaluate(async () => {
    await window.__MAZOCARTA_E2E__.waitReady();
  });
  await page.waitForFunction(() => {
    const state = window.__MAZOCARTA_E2E__?.getCombatHitCenters();
    return state && state.hand.length >= 2 && state.enemy.length >= 1;
  });
}

async function getCombatState(page) {
  return page.evaluate(() => ({
    frameCounter: window.__MAZOCARTA_E2E__.getFrameCounter(),
    hintText: window.__MAZOCARTA_E2E__.getHintText(),
    locked: window.__MAZOCARTA_E2E__.getCombatLockState(),
    hitCenters: window.__MAZOCARTA_E2E__.getCombatHitCenters(),
  }));
}

async function getStoredHostCombatSnapshot(page) {
  return page.evaluate(() => {
    const raw =
      window.localStorage.getItem("mazocarta.preview.active_run") ||
      window.localStorage.getItem("mazocarta.active_run");
    if (!raw) {
      return null;
    }
    const parsed = JSON.parse(raw);
    const combat = parsed?.active_state?.party_state?.combats?.[0];
    if (!combat) {
      return null;
    }
    return {
      enemyHp: combat.enemies?.map((enemy) => enemy?.fighter?.hp ?? null) || [],
      enemyBlock: combat.enemies?.map((enemy) => enemy?.fighter?.block ?? null) || [],
      hand: combat.deck?.hand || [],
      drawPile: combat.deck?.draw_pile || [],
      discardPile: combat.deck?.discard_pile || [],
      energy: combat.player?.energy ?? null,
      block: combat.player?.fighter?.block ?? null,
      phase: combat.phase ?? null,
      turn: combat.turn ?? null,
    };
  });
}

async function clickLogicalPoint(page, hitCenters, point) {
  const box = await page.locator("#game").boundingBox();
  expect(box).not.toBeNull();
  expect(point.x).toBeGreaterThan(0);
  expect(point.y).toBeGreaterThan(0);
  await page.mouse.click(
    box.x + (point.x / hitCenters.logicalWidth) * box.width,
    box.y + (point.y / hitCenters.logicalHeight) * box.height,
  );
}

test.beforeEach(async ({ page }) => {
  await openHostFirstCardFixture(page);
});

test("host first attack card does not freeze", async ({ page }) => {
  let state = await getCombatState(page);
  expect(state.hintText).toBe("Tap card or end turn");
  const beforeSnapshot = await getStoredHostCombatSnapshot(page);
  expect(beforeSnapshot, "stored host combat snapshot before attack").not.toBeNull();

  await clickLogicalPoint(page, state.hitCenters, state.hitCenters.hand[0]);
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
    .toBe("Tap enemy");

  state = await getCombatState(page);
  const frameStart = state.frameCounter;
  await clickLogicalPoint(page, state.hitCenters, state.hitCenters.enemy[0]);

  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
    .toBe("Resolving action...");
  await expect
    .poll(() =>
      page.evaluate((initial) => window.__MAZOCARTA_E2E__.getFrameCounter() - initial, frameStart),
    )
    .toBeGreaterThan(5);
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getCombatLockState()))
    .toBe(false);
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
    .not.toBe("Resolving encounter...");
  await expect
    .poll(async () => {
      const snapshot = await getStoredHostCombatSnapshot(page);
      return JSON.stringify(snapshot) !== JSON.stringify(beforeSnapshot);
    })
    .toBe(true);
  const afterSnapshot = await getStoredHostCombatSnapshot(page);
  expect(afterSnapshot, "stored host combat snapshot after attack").not.toBeNull();
  expect(afterSnapshot.enemyHp[0]).toBeLessThan(beforeSnapshot.enemyHp[0]);
  expect(afterSnapshot.hand.length).toBeLessThan(beforeSnapshot.hand.length);
  expect(afterSnapshot.energy).toBeLessThan(beforeSnapshot.energy);
});

test("host first defense card does not freeze", async ({ page }) => {
  let state = await getCombatState(page);
  expect(state.hintText).toBe("Tap card or end turn");
  const beforeSnapshot = await getStoredHostCombatSnapshot(page);
  expect(beforeSnapshot, "stored host combat snapshot before defense").not.toBeNull();

  await clickLogicalPoint(page, state.hitCenters, state.hitCenters.hand[1]);
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
    .toBe("Tap player");

  state = await getCombatState(page);
  const frameStart = state.frameCounter;
  await clickLogicalPoint(page, state.hitCenters, state.hitCenters.player);

  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
    .toBe("Resolving action...");
  await expect
    .poll(() =>
      page.evaluate((initial) => window.__MAZOCARTA_E2E__.getFrameCounter() - initial, frameStart),
    )
    .toBeGreaterThan(5);
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getCombatLockState()))
    .toBe(false);
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
    .not.toBe("Resolving encounter...");
  await expect
    .poll(async () => {
      const snapshot = await getStoredHostCombatSnapshot(page);
      return JSON.stringify(snapshot) !== JSON.stringify(beforeSnapshot);
    })
    .toBe(true);
  const afterSnapshot = await getStoredHostCombatSnapshot(page);
  expect(afterSnapshot, "stored host combat snapshot after defense").not.toBeNull();
  expect(afterSnapshot.block).toBeGreaterThan(beforeSnapshot.block);
  expect(afterSnapshot.hand.length).toBeLessThan(beforeSnapshot.hand.length);
  expect(afterSnapshot.energy).toBeLessThan(beforeSnapshot.energy);
});
