.DEFAULT_GOAL := all

.PHONY: all clean test build build-wasm serve debug publish-check enemy-previews

all: build

test:
	cargo test

build:
	./scripts/build-web.sh

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

clean:
	rm -rf target web/mazocarta.wasm web/.debug-mode.json web/icons web/apple-touch-icon.png
