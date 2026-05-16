import { chromium } from '/opt/homebrew/lib/node_modules/playwright/index.mjs';
import path from 'path';
import { fileURLToPath } from 'url';

const PREVIEW = encodeURI(`${process.env.HOME}/Library/Mobile Documents/com~apple~CloudDocs/claude-previews/hnr/preview.html`);
const OUTDIR = `${process.env.HOME}/Library/Mobile Documents/com~apple~CloudDocs/claude-previews/hnr`;

const browser = await chromium.launch();
const page = await browser.newPage();
await page.setViewportSize({ width: 1400, height: 900 });

await page.goto(`file://${PREVIEW}`);
await page.waitForTimeout(500);

// Full page overview
await page.screenshot({ path: `${OUTDIR}/hnr-overview.png`, fullPage: true });
console.log('Saved: hnr-overview.png');

// Individual screens — each .screen div
const screens = await page.$$('.screen');
const labels = ['01-top-stories', '02-comments-open', '03-command-mode', '04-new-feed'];

for (let i = 0; i < screens.length; i++) {
  const out = `${OUTDIR}/hnr-${labels[i]}.png`;
  await screens[i].screenshot({ path: out });
  console.log(`Saved: hnr-${labels[i]}.png`);
}

await browser.close();
console.log('Done.');
