# Third-Party Notices

This project generates browser-ready QR assets during build so the web client can run offline without a CDN.

- `web/qrcode.bundle.mjs` is generated from `qrcode` 1.5.4, MIT license.
- `web/jsqr.js` is generated from `jsqr` 1.4.0, Apache-2.0 license.

Regenerate both files with:

```bash
npm run vendor:qr
```

The generated files are ignored by Git. `package-lock.json` and `scripts/vendor-qr-libs.mjs` are the source of truth.
