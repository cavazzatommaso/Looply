<script lang="ts">
  import { FFmpeg } from "@ffmpeg/ffmpeg";
  import { fetchFile, toBlobURL } from "@ffmpeg/util";
  import { onDestroy, onMount } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { writable, type Writable } from "svelte/store";

  import { debounce } from "$lib/utils/debounce";
  import {
    getFilesFromPaths,
    factorFilesForUI,
    type UIFile,
    getImageSize,
  } from "$lib/utils/fileUtils";

  import DropzoneArea from "$lib/components/DropzoneArea.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import { checkForAppUpdates } from "$lib/utils/updater";

  // FFmpeg related state
  let ffmpeg: FFmpeg;
  let isLoading = false;
  let isReady = false;
  let isError = false;

  // File and GIF state
  let files: UIFile[] = [];
  let generatedGif: string = "";
  let outputUrl: string | null = null;

  // Configuration state
  let timeValue: Writable<number> = writable(0.5);
  let bitDepthValue: Writable<number> = writable(256);
  const KEY_time = "gif_time";
  const KEY_bit = "gif_bit";
  let isConfigSaved: boolean = false;

  // Tauri Drag & Drop state
  let isOver: boolean = false;
  let dropListener: UnlistenFn;
  let dragDropListener: Promise<UnlistenFn>;
  type DragDropPayload = {
    paths: string[];
    position: { x: number; y: number };
  };
  const baseURL = "https://unpkg.com/@ffmpeg/core@0.12.6/dist/esm";

  const debouncedCreateGif = debounce(() => {
    createGif();
  }, 1000);

  timeValue.subscribe(() => debouncedCreateGif());
  bitDepthValue.subscribe(() => debouncedCreateGif());

  onMount(async () => {
    isLoading = true;
    try {
      await checkForAppUpdates(false);
    } catch (error) {
      console.error(`Error updating: ${error}`);
    }
    const storedTime = localStorage.getItem(KEY_time);
    const storedBit = localStorage.getItem(KEY_bit);

    if (storedTime !== null) timeValue.set(Number(storedTime));
    if (storedBit !== null) bitDepthValue.set(Number(storedBit));

    listen('check-update', async () => {      
            await checkForAppUpdates(true);
    });

    dragDropListener = listen<DragDropPayload>(
      "tauri://drag-drop",
      async (event) => {
        const { paths } = event.payload;
        const rawFiles = await getFilesFromPaths(paths);
        const images = rawFiles.filter((file) =>
          file.type.startsWith("image/"),
        );
        addFiles(images);
      },
    );

    dropListener = await getCurrentWebview().onDragDropEvent((event) => {
      isOver = event.payload.type === "over";
    });

    ffmpeg = new FFmpeg();
    await loadffmpeg();
    isLoading = false;
    isReady = true;
  });

  onDestroy(async () => {
    dropListener();
    (await dragDropListener)();
    ffmpeg.terminate();
  });

  async function loadffmpeg() {
    // ffmpeg.on("log", ({ message }) => console.log(message));
    await ffmpeg.load({
      coreURL: await toBlobURL(`${baseURL}/ffmpeg-core.js`, "text/javascript"),
      wasmURL: await toBlobURL(
        `${baseURL}/ffmpeg-core.wasm`,
        "application/wasm",
      ),
    });
  }

  function saveConfig() {
    isConfigSaved = true;
    localStorage.setItem(KEY_time, $timeValue.toString());
    localStorage.setItem(KEY_bit, $bitDepthValue.toString());

    setTimeout(() => {
      isConfigSaved = false;
    }, 2000);
  }

  function addFiles(newFiles: File[]) {
    const factoredFiles = factorFilesForUI(newFiles);
    files = [...files, ...factoredFiles];
    createGif();
  }

  function deleteFile(fileId: string) {
    files = files.filter((file) => file.id !== fileId);
    createGif();
  }

  function reorderFiles(reorderedItems: UIFile[]) {
    files = reorderedItems;
  }

  async function createGif() {
    if (!isReady || files.length === 0) {
      // alert("FFmpeg not ready or no files.");
      return;
    }

    isLoading = true;
    isError = false;

    ffmpeg.terminate();
    await loadffmpeg();

    const tempDir = "/tmp";

    const sizes = await Promise.all(files.map((f) => getImageSize(f.file)));
    let maxWidth = 0;
    let maxHeight = 0;
    for (const size of sizes) {
      if (size.width > maxWidth) maxWidth = size.width;
      if (size.height > maxHeight) maxHeight = size.height;
    }

    for (const [i, file] of files.entries()) {
      const ext = file.file.name.split(".").pop() || "jpeg";
      const inputName = `${tempDir}/img${i}.${ext}`;
      await ffmpeg.writeFile(inputName, await fetchFile(file.file));

      const pngName = `${tempDir}/img${i}.png`;
      await ffmpeg.exec([
        "-i",
        inputName,
        "-vf",
        `scale=${maxWidth}:${maxHeight}:force_original_aspect_ratio=decrease,pad=${maxWidth}:${maxHeight}:(ow-iw)/2:(oh-ih)/2:color=black@0`,
        "-pix_fmt",
        "rgba",
        pngName,
      ]);
    }

    const fps = (1 / $timeValue).toFixed(2);
    const colors = $bitDepthValue;

    await ffmpeg.exec([
      "-framerate",
      `${fps}`,
      "-start_number",
      "0",
      "-i",
      `${tempDir}/img%d.png`,
      "-vf",
      `palettegen=max_colors=${colors}`,
      `${tempDir}/palette.png`,
    ]);

    try {
      await ffmpeg.exec([
        "-framerate",
        `${fps}`,
        "-start_number",
        "0",
        "-i",
        `${tempDir}/img%d.png`,
        "-i",
        `${tempDir}/palette.png`,
        "-lavfi",
        "paletteuse=dither=bayer:bayer_scale=4.4",
        "-loop",
        "0",
        "output.gif",
      ]);

      const data = await ffmpeg.readFile("output.gif");
      if (data.length === 0) throw new Error("Conversion failed");
      generatedGif = URL.createObjectURL(
        new Blob([data], { type: "image/gif" }),
      );
    } catch (error) {
      generatedGif = "";
      isError = true;
      console.error(error);
    }

    isLoading = false;
  }

  async function save_gif() {
    const selected = await save({
      defaultPath: "anim.gif",
      filters: [{ name: "GIF", extensions: ["gif"] }],
    });

    if (!selected) return;

    outputUrl = selected;

    const data = await ffmpeg.readFile("output.gif");

    await invoke("save_gif", {
      path: outputUrl,
      data: await new Blob([data], { type: "image/gif" }).arrayBuffer(),
    });

    alert("GIF created!");
  }
</script>

{#if isOver}
  <div class="h-screen w-screen bg-gray-200 flex justify-center items-center">
    Drop over the files
  </div>
{/if}

<div class="h-screen w-screen bg-white p-2">
  <div class="h-full w-full bg-gray-200 rounded-md grid grid-cols-2 gap-1 p-2">
    <DropzoneArea
      {files}
      addfiles={(files) => addFiles(files)}
      deletefile={(file) => deleteFile(file)}
      reorderfiles={(files) => reorderFiles(files)}
      dndcomplete={createGif}
    />

    <div class="h-full flex flex-col gap-2 p-2">
      <div class="text-xl font-mono text-center text-sky-500">export</div>
      <SettingsPanel
        {files}
        {timeValue}
        {bitDepthValue}
        {isConfigSaved}
        {isLoading}
        {isError}
        {generatedGif}
        {saveConfig}
        saveGif={save_gif}
      ></SettingsPanel>
    </div>
  </div>
</div>
