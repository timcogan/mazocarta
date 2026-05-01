import { expect, test } from "@playwright/test";

async function openE2EPage(page) {
  await page.goto("/?e2e=1");
  await page.evaluate(async () => {
    await window.__MAZOCARTA_E2E__.waitReady();
  });
}

async function captureFixtureSnapshot(browser, fixtureName) {
  const context = await browser.newContext();
  const page = await context.newPage();
  try {
    await openE2EPage(page);
    expect(await page.evaluate((name) => window.__MAZOCARTA_E2E__.loadFixture(name), fixtureName)).toBe(true);
    const snapshot = await page.evaluate(() => window.__MAZOCARTA_E2E__.getRunSnapshotRaw());
    expect(typeof snapshot).toBe("string");
    expect(snapshot.length).toBeGreaterThan(0);
    return snapshot;
  } finally {
    await context.close();
  }
}

async function getCombatState(page) {
  return page.evaluate(() => ({
    frameCounter: window.__MAZOCARTA_E2E__.getFrameCounter(),
    hintText: window.__MAZOCARTA_E2E__.getHintText(),
    locked: window.__MAZOCARTA_E2E__.getCombatLockState(),
    hitCenters: window.__MAZOCARTA_E2E__.getCombatHitCenters(),
  }));
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

async function scanAnimatedQrWithFakeCamera(
  sourcePage,
  targetPage,
  {
    intervalMs = 350,
    cameraSize = 448,
    mockDecode = false,
    expectPartialProgress = false,
    expectDetection = false,
    transformFrames = null,
  } = {},
) {
  let frames = await sourcePage.evaluate(() => window.__MAZOCARTA_E2E__.getPairQrFrames());
  const fullCode = await sourcePage.evaluate(() => window.__MAZOCARTA_E2E__.getPairCode());
  expect(Array.isArray(frames)).toBe(true);
  expect(frames.length).toBeGreaterThan(0);
  expect(typeof fullCode).toBe("string");
  expect(fullCode.length).toBeGreaterThan(0);
  if (typeof transformFrames === "function") {
    frames = transformFrames(frames, fullCode);
    expect(Array.isArray(frames)).toBe(true);
    expect(frames.length).toBeGreaterThan(0);
  }

  await expect(
    targetPage.evaluate(
      ({ qrFrames, qrIntervalMs, qrCameraSize, qrExposeFrameSource, qrMockDecode }) =>
        window.__MAZOCARTA_E2E__.installFakeQrCamera(qrFrames, {
          intervalMs: qrIntervalMs,
          size: qrCameraSize,
          forceJsQr: true,
          exposeFrameSource: qrExposeFrameSource,
          mockJsQr: qrMockDecode,
        }),
      {
        qrFrames: frames,
        qrIntervalMs: intervalMs,
        qrCameraSize: cameraSize,
        qrExposeFrameSource: true,
        qrMockDecode: mockDecode,
      },
    ),
  ).resolves.toBe(true);

  await expect
    .poll(() => targetPage.evaluate(() => window.__MAZOCARTA_E2E__.startRoomCameraScanner()))
    .toBe(true);
  await expect
    .poll(() => targetPage.evaluate(() => window.__MAZOCARTA_E2E__.isScannerActive()))
    .toBe(true);
  await expect
    .poll(() => targetPage.evaluate(() => window.__MAZOCARTA_E2E__.getRoomError()))
    .toBe("");

  if (expectPartialProgress && frames.length > 1) {
    await expect
      .poll(
        () =>
          targetPage.evaluate(
            () => window.__MAZOCARTA_E2E__.getTransportProgress()?.received ?? 0,
          ),
        { timeout: 12_000 },
      )
      .toBeGreaterThan(0);
    await expect
      .poll(
        () =>
          targetPage.evaluate(() => {
            const progress = document.querySelector("#multiplayer-scanner-progress");
            return progress instanceof HTMLElement && !progress.hidden;
          }),
        { timeout: 12_000 },
      )
      .toBe(true);
    await expect
      .poll(
        () =>
          targetPage.evaluate(
            () =>
              document.querySelector("#multiplayer-scanner-progress-label")?.textContent || "",
          ),
        { timeout: 12_000 },
      )
      .toContain("/");
  }

  if (expectDetection) {
    await expect
      .poll(
        () =>
          targetPage.evaluate(
            () => window.__MAZOCARTA_E2E__.getScannerDetection()?.widthPct ?? 0,
          ),
        { timeout: 12_000 },
      )
      .toBeGreaterThan(0);
  }

  await expect
    .poll(() => targetPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairManualInput()), {
      timeout: 20_000,
    })
    .toBe(fullCode);

  await expect(
    targetPage.evaluate(() => window.__MAZOCARTA_E2E__.clearFakeQrCamera()),
  ).resolves.toBe(true);

  return { frames, fullCode };
}

async function pairAndStartLanSession(
  browser,
  { hostContextOptions = {}, guestContextOptions = {} } = {},
) {
  const hostContext = await browser.newContext(hostContextOptions);
  const guestContext = await browser.newContext(guestContextOptions);
  const hostPage = await hostContext.newPage();
  const guestPage = await guestContext.newPage();

  try {
    await Promise.all([openE2EPage(hostPage), openE2EPage(guestPage)]);

    await hostPage.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoom());
    await guestPage.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoom());

    await expect
      .poll(() =>
        hostPage.evaluate(() => ({
          mode: window.__MAZOCARTA_E2E__.getRoomMode(),
          codeLen: window.__MAZOCARTA_E2E__.getPairCode().length,
        })),
      )
      .toMatchObject({ mode: "host-room" });

    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairCode().length))
      .toBeGreaterThan(0);

    const hostOffer = await hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairCode());
    await guestPage.evaluate((code) => window.__MAZOCARTA_E2E__.applyPairCode(code), hostOffer);

    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getRoomMode()))
      .toBe("guest-confirm");
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairCode().length))
      .toBeGreaterThan(0);

    const guestAnswer = await guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairCode());
    await hostPage.evaluate((code) => window.__MAZOCARTA_E2E__.applyPairCode(code), guestAnswer);

    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getRoomMode()))
      .toBe("guest-waiting");
    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getParticipants().length))
      .toBe(2);

    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.startHostRun()))
      .toBe(true);
    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.isSessionStarted()))
      .toBe(true);
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.isSessionStarted()))
      .toBe(true);

    return { hostContext, guestContext, hostPage, guestPage };
  } catch (error) {
    await hostContext.close();
    await guestContext.close();
    throw error;
  }
}

async function pairAndStartLanSessionByFakeCamera(browser) {
  const hostContext = await browser.newContext();
  const guestContext = await browser.newContext();
  const hostPage = await hostContext.newPage();
  const guestPage = await guestContext.newPage();

  try {
    await Promise.all([openE2EPage(hostPage), openE2EPage(guestPage)]);

    await hostPage.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoomQr());
    await guestPage.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoomQr());

    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairCode().length))
      .toBeGreaterThan(0);

    const { fullCode: hostOffer } = await scanAnimatedQrWithFakeCamera(hostPage, guestPage, {
      mockDecode: true,
    });
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getRoomMode()))
      .toBe("guest-confirm");
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairCode().length))
      .toBeGreaterThan(0);

    const { fullCode: guestAnswer } = await scanAnimatedQrWithFakeCamera(guestPage, hostPage, {
      mockDecode: true,
    });
    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairManualInput()))
      .toBe(guestAnswer);
    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getParticipants().length))
      .toBe(2);
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairManualInput()))
      .toBe(hostOffer);
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getRoomMode()))
      .toBe("guest-waiting");

    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.startHostRun()))
      .toBe(true);
    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.isSessionStarted()))
      .toBe(true);
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.isSessionStarted()))
      .toBe(true);

    return { hostContext, guestContext, hostPage, guestPage };
  } catch (error) {
    await hostContext.close();
    await guestContext.close();
    throw error;
  }
}

async function expectMultiplayerButtonsUseUiKit(page) {
  const buttons = await page.locator("#multiplayer-ui button").evaluateAll((elements) =>
    elements.map((element) => ({
      text: element.textContent?.trim() || "",
      classes: Array.from(element.classList),
    })),
  );
  expect(buttons.length).toBeGreaterThan(0);
  for (const button of buttons) {
    expect(button.classes, `${button.text} should use uiButton`).toContain("ui-button");
    expect(
      button.classes.some((className) =>
        ["ui-button--primary", "ui-button--secondary", "ui-button--danger"].includes(className),
      ),
      `${button.text} should use a uiButton variant`,
    ).toBe(true);
  }
}

async function expectWideButtonRow(page, expectedActions) {
  const rows = await page.locator("#multiplayer-ui .ui-button-row").evaluateAll((elements) =>
    elements.map((row) => {
      const rowRect = row.getBoundingClientRect();
      const buttons = Array.from(row.querySelectorAll(":scope > button.ui-button")).map(
        (button) => {
          const rect = button.getBoundingClientRect();
          const style = getComputedStyle(button);
          return {
            action: button.dataset.action || "",
            flexGrow: style.flexGrow,
            width: rect.width,
          };
        },
      );
      return {
        actions: buttons.map((button) => button.action),
        buttons,
        classes: Array.from(row.classList),
        width: rowRect.width,
      };
    }),
  );
  const row = rows.find((candidate) =>
    expectedActions.every((action) => candidate.actions.includes(action)),
  );
  expect(row, `button row for ${expectedActions.join(", ")}`).toBeTruthy();
  expect(row.classes).toContain("ui-button-row--grow");
  for (const button of row.buttons) {
    expect(button.flexGrow, `${button.action} should grow`).toBe("1");
    expect(button.width, `${button.action} should be wide`).toBeGreaterThan(row.width * 0.35);
  }
}

test("host and guest can pair and start a LAN session by code", async ({ browser }) => {
  const session = await pairAndStartLanSession(browser);
  try {
    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("opening_intro");
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("opening_intro");
  } finally {
    await session.hostContext.close();
    await session.guestContext.close();
  }
});

test("active host returns directly to the run from the multiplayer button", async ({ browser }) => {
  const session = await pairAndStartLanSession(browser);
  try {
    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("opening_intro");

    expect(await session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.returnToMenu())).toBe(
      true,
    );
    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.isBootScreen()))
      .toBe(true);
    await session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.openMultiplayerEntry());
    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("opening_intro");
    expect(
      await session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.isMultiplayerRoomOpen()),
    ).toBe(false);
  } finally {
    await session.hostContext.close();
    await session.guestContext.close();
  }
});

test("multiplayer modal buttons reuse ui-kit variants", async ({ page }) => {
  await openE2EPage(page);

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openMultiplayerEntry());
  await expectMultiplayerButtonsUseUiKit(page);
  await expect(page.locator('#multiplayer-ui button[data-action="leave-session"]')).toHaveText(
    "Back",
  );
  await expect(page.locator('#multiplayer-ui button[data-action="leave-session"]')).toHaveClass(
    /ui-button--secondary/,
  );

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoomQr());
  await expectMultiplayerButtonsUseUiKit(page);

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoomQr());
  await expectMultiplayerButtonsUseUiKit(page);
});

test("multiplayer compact action rows use wide button layout", async ({ page }) => {
  await openE2EPage(page);

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoomQr());
  await expectWideButtonRow(page, ["copy-local-code", "refresh-host-invite"]);
  await page.getByRole("button", { name: "Paste code" }).click();
  await expectWideButtonRow(page, ["apply-remote-code"]);

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoomQr());
  await page.evaluate(() =>
    window.__MAZOCARTA_E2E__.installFakeQrCamera(["not-a-mazocarta-code"], {
      intervalMs: 1000,
      forceJsQr: true,
    }),
  );
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.startRoomCameraScanner()))
    .toBe(true);
  await expectWideButtonRow(page, ["reset-scanner", "toggle-scanner-facing"]);
  await page.getByRole("button", { name: "Paste code" }).click();
  await expectWideButtonRow(page, ["apply-remote-code"]);
  await page.evaluate(() => window.__MAZOCARTA_E2E__.clearFakeQrCamera());
});

test("host disconnected main menu action uses in-canvas game button layout on mobile", async ({
  browser,
}) => {
  test.setTimeout(30_000);
  const session = await pairAndStartLanSession(browser, {
    guestContextOptions: {
      viewport: { width: 390, height: 720 },
      isMobile: true,
      hasTouch: true,
    },
  });
  try {
    await session.hostContext.close();
    session.hostContext = null;

    await expect
      .poll(
        () =>
          session.guestPage.evaluate(
            () => window.__MAZOCARTA_E2E__.getBlockingScreen()?.title,
          ),
        { timeout: 15_000 },
      )
      .toBe("Host Disconnected");

    const { rect, size } = await session.guestPage.evaluate(() => ({
      rect: window.__MAZOCARTA_E2E__.getBlockingScreenActionRect(),
      size: window.__MAZOCARTA_E2E__.getLogicalSize(),
    }));
    expect(rect).not.toBeNull();
    expect(rect.width).toBeGreaterThan(0);
    expect(rect.height).toBeGreaterThan(0);
    expect(rect.x).toBeGreaterThanOrEqual(0);
    expect(rect.y).toBeGreaterThanOrEqual(0);
    expect(rect.x + rect.width).toBeLessThanOrEqual(size.width + 0.5);
    expect(rect.y + rect.height).toBeLessThanOrEqual(size.height + 0.5);

    await clickLogicalPoint(
      session.guestPage,
      { logicalWidth: size.width, logicalHeight: size.height },
      {
        x: rect.x + rect.width * 0.5,
        y: rect.y + rect.height * 0.5,
      },
    );
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.isBootScreen()), {
        timeout: 5_000,
      })
      .toBe(true);
  } finally {
    if (session.hostContext) {
      await session.hostContext.close();
    }
    await session.guestContext.close();
  }
});

test("multiplayer modal outer border is invisible", async ({ page }) => {
  await openE2EPage(page);
  await page.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoom());

  const borderColor = await page
    .locator("#multiplayer-ui .ui-card.multiplayer-modal-card")
    .first()
    .evaluate((element) => getComputedStyle(element).borderColor);

  expect(borderColor).toBe("rgba(0, 0, 0, 0)");
});

test("multiplayer room cards keep uniform content padding", async ({ page }) => {
  await openE2EPage(page);

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoomQr());
  let padding = await page.locator("#multiplayer-ui .multiplayer-room-card").evaluate((element) => {
    const style = getComputedStyle(element);
    return [style.paddingTop, style.paddingRight, style.paddingBottom, style.paddingLeft];
  });
  expect(new Set(padding).size).toBe(1);

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoomQr());
  padding = await page.locator("#multiplayer-ui .multiplayer-guest-card").evaluate((element) => {
    const style = getComputedStyle(element);
    return [style.paddingTop, style.paddingRight, style.paddingBottom, style.paddingLeft];
  });
  expect(new Set(padding).size).toBe(1);
});

test("scanner defaults to the back camera", async ({ page }) => {
  await openE2EPage(page);
  await page.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoomQr());

  expect(await page.evaluate(() => window.__MAZOCARTA_E2E__.getCameraFacing())).toBe(
    "environment",
  );
});

test("active scanner uses reset instead of stop", async ({ page }) => {
  await openE2EPage(page);
  await page.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoomQr());
  await page.evaluate(() =>
    window.__MAZOCARTA_E2E__.installFakeQrCamera(["not-a-mazocarta-code"], {
      intervalMs: 1000,
      forceJsQr: true,
    }),
  );

  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.startRoomCameraScanner()))
    .toBe(true);

  await expect(page.locator('#multiplayer-ui button[data-action="reset-scanner"]')).toHaveText(
    "Reset",
  );
  await expect(page.locator("#multiplayer-ui")).not.toContainText("Stop camera");

  await page.locator('#multiplayer-ui button[data-action="reset-scanner"]').click();
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.isScannerActive()))
    .toBe(true);
});

test("scanner reset clears an incomplete QR frame assembly", async ({ browser }) => {
  const hostContext = await browser.newContext();
  const guestContext = await browser.newContext();
  const hostPage = await hostContext.newPage();
  const guestPage = await guestContext.newPage();
  try {
    await Promise.all([openE2EPage(hostPage), openE2EPage(guestPage)]);
    await hostPage.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoomQr());
    await guestPage.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoomQr());

    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairQrFrames().length))
      .toBeGreaterThan(2);

    const frames = await hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairQrFrames());
    const incompleteFrames = frames.slice(0, -1);
    await guestPage.evaluate(
      ({ qrFrames }) =>
        window.__MAZOCARTA_E2E__.installFakeQrCamera(qrFrames, {
          intervalMs: 500,
          forceJsQr: true,
        }),
      { qrFrames: incompleteFrames },
    );

    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.startRoomCameraScanner()))
      .toBe(true);
    await expect
      .poll(
        () =>
          guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getTransportProgress()?.received ?? 0),
        { timeout: 12_000 },
      )
      .toBe(incompleteFrames.length);
    await expect
      .poll(() =>
        guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getTransportProgress()?.total ?? 0),
      )
      .toBe(frames.length);

    await guestPage.locator('#multiplayer-ui button[data-action="reset-scanner"]').click();
    await expect
      .poll(
        () =>
          guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getTransportProgress()?.received ?? 0),
        { timeout: 2_000 },
      )
      .toBeLessThan(incompleteFrames.length);
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.isScannerActive()))
      .toBe(true);
  } finally {
    await hostContext.close();
    await guestContext.close();
  }
});

test("multiplayer room copy stays concise for host and join", async ({ page }) => {
  await openE2EPage(page);

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoomQr());
  let text = await page.locator("#multiplayer-ui").innerText();
  expect(text).toContain("Show QR to guests");
  for (const forbidden of [
    "Connection Room",
    "Show this code to the guest",
    "Pair players here",
    "Animated QR",
    "loops automatically",
    "Keep it steady",
  ]) {
    expect(text).not.toContain(forbidden);
  }

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoomQr());
  text = await page.locator("#multiplayer-ui").innerText();
  for (const forbidden of [
    "Join the host here",
    "After you scan it",
    "Animated QR",
    "loops automatically",
  ]) {
    expect(text).not.toContain(forbidden);
  }
});

test("connection room is top aligned on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 740 });
  await openE2EPage(page);
  await page.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoomQr());

  const alignItems = await page
    .locator("#multiplayer-ui .multiplayer-room-shell")
    .evaluate((element) => getComputedStyle(element).alignItems);
  const box = await page.locator("#multiplayer-ui .multiplayer-room-card").boundingBox();

  expect(alignItems).toBe("flex-start");
  expect(box).not.toBeNull();
  expect(box.y).toBeLessThanOrEqual(24);
});

test("entry Back ignores immediate accidental tap after opening", async ({ page }) => {
  await openE2EPage(page);
  await page.evaluate(() => window.__MAZOCARTA_E2E__.openMultiplayerEntry());
  await expect(page.getByRole("button", { name: "Back" })).toBeVisible();

  await page.evaluate(() => {
    document.querySelector('#multiplayer-ui [data-action="leave-session"]')?.click();
  });
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.isMultiplayerRoomOpen()))
    .toBe(true);

  await page.waitForTimeout(500);
  await page.getByRole("button", { name: "Back" }).click();
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.isMultiplayerRoomOpen()))
    .toBe(false);
});

test("host offers resume or new run when a save exists", async ({ browser, page }) => {
  const snapshot = await captureFixtureSnapshot(browser, "multiplayer-map");
  await openE2EPage(page);
  expect(
    await page.evaluate((raw) => window.__MAZOCARTA_E2E__.storeRunSnapshot(raw), snapshot),
  ).toBe(true);

  await page.evaluate(() => window.__MAZOCARTA_E2E__.openMultiplayerEntry());
  await page.getByRole("button", { name: "Host" }).click();

  await expect(page.getByRole("button", { name: "Resume Saved Run" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Start New Run" })).toBeVisible();

  await page.getByRole("button", { name: "Resume Saved Run" }).click();
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getRoomMode()))
    .toBe("host-room");
  await expect
    .poll(() => page.evaluate(() => window.__MAZOCARTA_E2E__.getRoomError()))
    .toBe("");
});

test("guest scanner ignores noisy QR frames without resetting progress", async ({ browser }) => {
  const hostContext = await browser.newContext();
  const guestContext = await browser.newContext();
  const hostPage = await hostContext.newPage();
  const guestPage = await guestContext.newPage();
  try {
    await Promise.all([openE2EPage(hostPage), openE2EPage(guestPage)]);
    await hostPage.evaluate(() => window.__MAZOCARTA_E2E__.openHostRoomQr());
    await guestPage.evaluate(() => window.__MAZOCARTA_E2E__.openGuestRoomQr());

    await expect
      .poll(() => hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairCode().length))
      .toBeGreaterThan(0);

    const { fullCode: hostOffer } = await scanAnimatedQrWithFakeCamera(hostPage, guestPage, {
      intervalMs: 360,
      expectPartialProgress: true,
      expectDetection: true,
      transformFrames: (frames) => {
        const noisyFrames = [];
        for (const [index, frame] of frames.entries()) {
          noisyFrames.push(frame);
          if (index === 0) {
            noisyFrames.push(frame);
          }
          noisyFrames.push("MZQ1:malformed");
        }
        return noisyFrames;
      },
    });

    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPairManualInput()))
      .toBe(hostOffer);
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getRoomError()))
      .toBe("");
    await expect
      .poll(() => guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getRoomMode()))
      .toBe("guest-confirm");
  } finally {
    await hostContext.close();
    await guestContext.close();
  }
});

test("host and guest can pair and start a LAN session through the fake-camera scanner path", async ({
  browser,
}) => {
  test.setTimeout(40_000);
  const session = await pairAndStartLanSessionByFakeCamera(browser);
  try {
    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("opening_intro");
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("opening_intro");
  } finally {
    await session.hostContext.close();
    await session.guestContext.close();
  }
});

test("guest sees the map with a waiting banner while host controls it", async ({ browser }) => {
  const session = await pairAndStartLanSession(browser);
  try {
    expect(await session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.loadFixture("multiplayer-map"))).toBe(
      true,
    );

    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("map");
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("map");

    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getBlockingScreen()))
      .toBe(null);
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getBlockingScreen()))
      .toMatchObject({
        title: "Waiting on host",
        presentation: "banner",
      });
    const { rect, size } = await session.guestPage.evaluate(() => ({
      rect: window.__MAZOCARTA_E2E__.getBlockingScreenBannerRect(),
      size: window.__MAZOCARTA_E2E__.getLogicalSize(),
    }));
    expect(rect).not.toBeNull();
    expect(Math.abs(rect.y + rect.height * 0.5 - size.height * 0.5)).toBeLessThan(1);
  } finally {
    await session.hostContext.close();
    await session.guestContext.close();
  }
});

test("guest combat play does not get stuck on resolving", async ({ browser }) => {
  const combatSnapshot = await captureFixtureSnapshot(browser, "host-first-card");
  const session = await pairAndStartLanSession(browser);
  try {
    expect(
      await session.hostPage.evaluate((raw) => window.__MAZOCARTA_E2E__.restoreSnapshot(raw), combatSnapshot),
    ).toBe(true);

    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("combat");
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("combat");
    await session.guestPage.waitForFunction(() => {
      const state = window.__MAZOCARTA_E2E__?.getCombatHitCenters();
      return state && state.hand.length >= 2 && state.enemy.length >= 1;
    });

    let state = await getCombatState(session.guestPage);
    expect(state.hintText).toBe("Tap card or end turn");

    await clickLogicalPoint(session.guestPage, state.hitCenters, state.hitCenters.hand[0]);
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
      .toBe("Tap enemy");

    state = await getCombatState(session.guestPage);
    const frameStart = state.frameCounter;
    await clickLogicalPoint(session.guestPage, state.hitCenters, state.hitCenters.enemy[0]);

    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
      .toBe("Resolving action...");
    await expect
      .poll(() =>
        session.guestPage.evaluate(
          (initial) => window.__MAZOCARTA_E2E__.getFrameCounter() - initial,
          frameStart,
        ),
      )
      .toBeGreaterThan(5);
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getCombatLockState()))
      .toBe(false);
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
      .not.toBe("Resolving encounter...");
  } finally {
    await session.hostContext.close();
    await session.guestContext.close();
  }
});

test("guest rejected combat action recovers while host playback is active", async ({ browser }) => {
  const combatSnapshot = await captureFixtureSnapshot(browser, "host-first-card");
  const session = await pairAndStartLanSession(browser);
  try {
    expect(
      await session.hostPage.evaluate((raw) => window.__MAZOCARTA_E2E__.restoreSnapshot(raw), combatSnapshot),
    ).toBe(true);

    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("combat");
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("combat");

    await session.hostPage.waitForFunction(() => {
      const state = window.__MAZOCARTA_E2E__?.getCombatHitCenters();
      return state && state.hand.length >= 2 && state.enemy.length >= 1;
    });
    await session.guestPage.waitForFunction(() => {
      const state = window.__MAZOCARTA_E2E__?.getCombatHitCenters();
      return state && state.hand.length >= 2 && state.enemy.length >= 1;
    });

    let hostState = await getCombatState(session.hostPage);
    await clickLogicalPoint(session.hostPage, hostState.hitCenters, hostState.hitCenters.hand[0]);
    hostState = await getCombatState(session.hostPage);
    await clickLogicalPoint(session.hostPage, hostState.hitCenters, hostState.hitCenters.enemy[0]);

    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
      .toBe("Resolving action...");

    let guestState = await getCombatState(session.guestPage);
    await clickLogicalPoint(session.guestPage, guestState.hitCenters, guestState.hitCenters.hand[0]);
    guestState = await getCombatState(session.guestPage);
    await clickLogicalPoint(session.guestPage, guestState.hitCenters, guestState.hitCenters.enemy[0]);

    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getCombatLockState()))
      .toBe(false);
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getHintText()))
      .not.toBe("Resolving encounter...");
  } finally {
    await session.hostContext.close();
    await session.guestContext.close();
  }
});

test("reward barrier stays authoritative until the host snapshot advances both players", async ({
  browser,
}) => {
  const rewardSnapshot = await captureFixtureSnapshot(browser, "multiplayer-reward-barrier");
  const session = await pairAndStartLanSession(browser);
  try {
    expect(
      await session.hostPage.evaluate(
        (raw) => window.__MAZOCARTA_E2E__.restoreSnapshot(raw),
        rewardSnapshot,
      ),
    ).toBe(true);

    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("reward");
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("reward");

    const generationBeforeHostChoice = await session.hostPage.evaluate(() =>
      window.__MAZOCARTA_E2E__.getRunSaveGeneration(),
    );
    await session.hostPage.keyboard.press("1");

    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getRunSaveGeneration()))
      .toBeGreaterThan(generationBeforeHostChoice);
    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("reward");
    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getBlockingScreen()))
      .toMatchObject({
        title: "Waiting on players",
      });
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("reward");

    const guestBlockingAfterHostChoice = await session.guestPage.evaluate(() =>
      window.__MAZOCARTA_E2E__.getBlockingScreen(),
    );
    expect(guestBlockingAfterHostChoice?.title ?? null).not.toBe("Waiting on host");

    const generationBeforeGuestChoice = await session.hostPage.evaluate(() =>
      window.__MAZOCARTA_E2E__.getRunSaveGeneration(),
    );
    await session.guestPage.keyboard.press("1");

    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getRunSaveGeneration()))
      .toBeGreaterThan(generationBeforeGuestChoice);
    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("map");
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getPartyScreen()))
      .toBe("map");
    await expect
      .poll(() => session.hostPage.evaluate(() => window.__MAZOCARTA_E2E__.getBlockingScreen()))
      .toBe(null);
    await expect
      .poll(() => session.guestPage.evaluate(() => window.__MAZOCARTA_E2E__.getBlockingScreen()))
      .toMatchObject({
        title: "Waiting on host",
        presentation: "banner",
      });
  } finally {
    await session.hostContext.close();
    await session.guestContext.close();
  }
});
