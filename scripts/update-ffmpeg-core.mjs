import fs from "fs";
import path from "path";
import { Readable } from 'stream';
import { finished } from 'stream/promises';

async function downloadFile(url, dest) {
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`Failed to fetch ${url}: ${res.status} ${res.statusText}`);
  }
  const fileStream = fs.createWriteStream(dest);
  await finished(Readable.fromWeb(res.body).pipe(fileStream));
  console.log(`File downloaded to ${dest}`);
}

async function main() {
  const registryUrl = "https://registry.npmjs.org/@ffmpeg/core";
  const res = await fetch(registryUrl);
  if (!res.ok) throw new Error(`Failed to fetch registry: ${res.statusText}`);
  
  const data = await res.json();
  const latestVersion = data["dist-tags"]?.latest;
  if (!latestVersion) throw new Error("Latest version not found");
  console.log(`Latest FFmpeg version: ${latestVersion}`);

  const configPath = path.resolve("src/lib/assets/ffmpeg-version.json");
  let localVersion = null;
  if (fs.existsSync(configPath)) {
    try {
      const raw = fs.readFileSync(configPath, "utf-8");
      localVersion = JSON.parse(raw).version;
    } catch {
      console.warn("Failed to parse local version JSON");
    }
  }

  if (localVersion === latestVersion) {
    console.log(`Already up to date (${localVersion})`);
    return;
  }

  console.log(`Updating from ${localVersion ?? "none"} to ${latestVersion}`);

  const outDir = path.resolve("static/ffmpeg");
  
  if (fs.existsSync(outDir)) {
    fs.rmSync(outDir, { recursive: true, force: true });
  }
  fs.mkdirSync(outDir, { recursive: true });

  const baseUrl = `https://unpkg.com/@ffmpeg/core@${latestVersion}/dist/esm`;
  await Promise.all([
    downloadFile(`${baseUrl}/ffmpeg-core.js`, path.join(outDir, "ffmpeg-core.js")),
    downloadFile(`${baseUrl}/ffmpeg-core.wasm`, path.join(outDir, "ffmpeg-core.wasm")),
  ]);

  fs.writeFileSync(configPath, JSON.stringify({ version: latestVersion }, null, 2));
  console.log("FFmpeg core update complete!");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});