import { join } from "node:path";
import sharp from "sharp";

const PUBLIC = join(import.meta.dirname, "..", "public");

const logoSvg = (color, size) => `
<svg width="${size}" height="${size}" viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
  <circle cx="50" cy="50" r="45" stroke="${color}" stroke-width="6" fill="none"/>
  <circle cx="50" cy="50" r="35" stroke="${color}" stroke-width="3" fill="none"/>
  <text x="50" y="58" text-anchor="middle" fill="${color}" font-size="36" font-weight="bold" font-family="system-ui, sans-serif">P</text>
</svg>`;

const logoWithBg = (fgColor, bgColor, size) => `
<svg width="${size}" height="${size}" viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
  <rect width="100" height="100" fill="${bgColor}"/>
  <circle cx="50" cy="50" r="40" stroke="${fgColor}" stroke-width="5" fill="none"/>
  <circle cx="50" cy="50" r="31" stroke="${fgColor}" stroke-width="2.5" fill="none"/>
  <text x="50" y="57" text-anchor="middle" fill="${fgColor}" font-size="32" font-weight="bold" font-family="system-ui, sans-serif">P</text>
</svg>`;

const orange = "#f97316";
const darkBg = "#0a0a0a";

async function generate() {
  // Transparent favicons
  for (const size of [16, 32]) {
    await sharp(Buffer.from(logoSvg(orange, 100)))
      .resize(size, size)
      .png()
      .toFile(join(PUBLIC, `favicon-${size}x${size}.png`));
  }

  // favicon.ico (32x32 PNG renamed — browsers accept PNG in .ico)
  await sharp(Buffer.from(logoSvg(orange, 100)))
    .resize(32, 32)
    .png()
    .toFile(join(PUBLIC, "favicon.ico"));

  // Apple touch icon (180x180, solid background)
  await sharp(Buffer.from(logoWithBg(orange, darkBg, 100)))
    .resize(180, 180)
    .png()
    .toFile(join(PUBLIC, "apple-touch-icon.png"));

  // Android chrome icons (solid background)
  for (const size of [192, 512]) {
    await sharp(Buffer.from(logoWithBg(orange, darkBg, 100)))
      .resize(size, size)
      .png()
      .toFile(join(PUBLIC, `android-chrome-${size}x${size}.png`));
  }

  // MS tile (150x150, solid background)
  await sharp(Buffer.from(logoWithBg(orange, darkBg, 100)))
    .resize(150, 150)
    .png()
    .toFile(join(PUBLIC, "mstile-150x150.png"));

  console.log("Favicons generated successfully.");
}

generate().catch(console.error);
