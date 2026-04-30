.DEFAULT_GOAL := all

.PHONY: all clean test test-e2e soak-2p build build-e2e build-wasm serve debug publish-check enemy-previews \
	android-sync android-setup-sdk android-build android-install android-run

all: build

test:
	cargo test

test-e2e: build-e2e
	npm run test:e2e

soak-2p: build-e2e
	npm run soak:2p

build:
	./scripts/build-web.sh

build-e2e:
	MAZOCARTA_CARGO_FEATURES=e2e ./scripts/build-web.sh

build-wasm: build

serve:
	@rm -f web/.debug-mode.json
	python3 -m http.server 4173 --directory web

debug: build
	@printf '{ "enabled": true }\n' > web/.debug-mode.json; \
	trap 'rm -f web/.debug-mode.json' EXIT INT TERM; \
	echo "Open http://localhost:4173/"; \
	python3 -m http.server 4173 --directory web

publish-check:
	./scripts/publish-check.sh

enemy-previews:
	python3 scripts/render-enemy-previews.py

android-sync: build
	./scripts/android-sync-assets.sh

android-setup-sdk:
	./scripts/setup-android-sdk.sh

android-build:
	./scripts/android-gradle.sh assembleDebug

android-install:
	./scripts/android-gradle.sh installDebug

android-run:
	adb shell am start -n com.mazocarta.android.debug/com.mazocarta.android.MainActivity

clean:
	rm -rf target web/mazocarta.wasm web/jsqr.js web/qrcode.bundle.mjs web/.debug-mode.json web/icons web/apple-touch-icon.png android/app/src/main/assets/site android/app/src/main/res/drawable/ic_launcher.png android/app/src/main/res/drawable/ic_launcher_foreground.xml android/app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml android/app/src/main/res/mipmap-anydpi-v26/ic_launcher_round.xml .gradle-bootstrap
