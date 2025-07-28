<script lang="ts">
  import { FFmpeg } from "@ffmpeg/ffmpeg";
  import { fetchFile } from "@ffmpeg/util";
  import ffmpegConfig from "$lib/assets/ffmpeg-version.json";
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
  import { RotateCcw } from "lucide-svelte";

  // FFmpeg related state
  let ffmpeg: FFmpeg;
  let isLoading = false;
  let isReady = false;
  let isError = false;
  let progressStatus: string = "";
  let progressPercent: number = 0;

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

  const baseURL = `/ffmpeg/${ffmpegConfig.version}`;

  const debouncedCreateGif = debounce(() => {
    createGif();
  }, 1000);

  timeValue.subscribe(() => debouncedCreateGif());
  bitDepthValue.subscribe(() => debouncedCreateGif());

  onMount(async () => {
    isLoading = true;
    progressStatus = "Checking for updates...";
    progressPercent = 25;
    try {
      await checkForAppUpdates(false);
    } catch (error) {
      console.error(`Error updating: ${error}`);
    }
    progressStatus = "Loading saved config...";
    progressPercent = 50;
    const storedTime = localStorage.getItem(KEY_time);
    const storedBit = localStorage.getItem(KEY_bit);

    if (storedTime !== null) timeValue.set(Number(storedTime));
    if (storedBit !== null) bitDepthValue.set(Number(storedBit));

    listen("check-update", async () => {
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

    progressStatus = "Load FFmpeg...";
    progressPercent = 75;

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
      coreURL: `${baseURL}/ffmpeg-core.js`,
      wasmURL: `${baseURL}/ffmpeg-core.wasm`,
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

  function reset() {
    files = [];
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
    progressStatus = "Preparing FFmpeg...";
    progressPercent = 0;

    try {
      ffmpeg.terminate();
      await loadffmpeg();

      progressStatus = "Analyzing images...";
      progressPercent = 10;

      const tempDir = "/tmp";
      const totalSteps = files.length + 3; // +3 for palette generation, gif creation, and cleanup
      let currentStep = 1;

      const sizes = await Promise.all(files.map((f) => getImageSize(f.file)));
      let maxWidth = 0;
      let maxHeight = 0;
      for (const size of sizes) {
        if (size.width > maxWidth) maxWidth = size.width;
        if (size.height > maxHeight) maxHeight = size.height;
      }

      progressStatus = "Processing images...";
      progressPercent = 20;
      const pngFileNames = [];

      for (const [i, file] of files.entries()) {
        progressStatus = `Processing image ${i + 1} of ${files.length}...`;
        progressPercent = 20 + (currentStep / totalSteps) * 50;
        const ext = file.file.name.split(".").pop() || "jpeg";
        const inputName = `${tempDir}/img_original_${i}.${ext}`;
        const pngName = `${tempDir}/img${i}.png`;
        pngFileNames.push(pngName);

        try {
          await ffmpeg.writeFile(inputName, await fetchFile(file.file));

          await ffmpeg.exec([
            "-i",
            inputName,
            "-vf",
            `scale=${maxWidth}:${maxHeight}:force_original_aspect_ratio=decrease,pad=${maxWidth}:${maxHeight}:(ow-iw)/2:(oh-ih)/2:color=black@0`,
            "-pix_fmt",
            "rgba",
            pngName,
          ]);
        } catch (e) {
          console.error("Error processing image:", file.file.name, e);
          isError = true;
          isLoading = false;
          progressStatus = "";
          progressPercent = 0;
          return;
        } finally {
          try {
            await ffmpeg.deleteFile(inputName);
          } catch (delError) {
            console.warn(`Could not delete ${inputName}:`, delError);
          }
        }
        currentStep++;
      }

      const fps = (1 / $timeValue).toFixed(2);
      const colors = $bitDepthValue;
      const paletteName = `${tempDir}/palette.png`;

      progressStatus = "Generating color palette...";
      progressPercent = 75;

      try {
        await ffmpeg.exec([
          "-framerate",
          `${fps}`,
          "-start_number",
          "0",
          "-i",
          `${tempDir}/img%d.png`,
          "-vf",
          `palettegen=max_colors=${colors}`,
          paletteName,
        ]);

        progressStatus = "Creating GIF...";
        progressPercent = 85;

        await ffmpeg.exec([
          "-framerate",
          `${fps}`,
          "-start_number",
          "0",
          "-i",
          `${tempDir}/img%d.png`,
          "-i",
          paletteName,
          "-lavfi",
          "paletteuse=dither=bayer:bayer_scale=4.4",
          "-loop",
          "0",
          "output.gif",
        ]);

        progressStatus = "Finalizing...";
        progressPercent = 95;

        const data = await ffmpeg.readFile("output.gif");
        if (data.length === 0) throw new Error("Conversion failed");
        generatedGif = URL.createObjectURL(
          new Blob([data], { type: "image/gif" }),
        );

        progressStatus = "Complete!";
        progressPercent = 100;
      } catch (error) {
        generatedGif = "";
        isError = true;
        progressStatus = "";
        progressPercent = 0;
        console.error("FFmpeg conversion error:", error);
      } finally {
        for (const pngFile of pngFileNames) {
          try {
            await ffmpeg.deleteFile(pngFile);
          } catch (delError) {
            console.warn(`Could not delete ${pngFile}:`, delError);
          }
        }
        try {
          await ffmpeg.deleteFile(paletteName);
        } catch (delError) {
          console.warn(`Could not delete ${paletteName}:`, delError);
        }
      }
    } catch (error) {
      isError = true;
      progressStatus = "";
      progressPercent = 0;
      console.error("Unexpected error:", error);
    } finally {
      isLoading = false;
      // Reset progress after a short delay to show completion
      setTimeout(() => {
        if (!isLoading) {
          progressStatus = "";
          progressPercent = 0;
        }
      }, 1000);
    }
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
    {#if files.length > 0}
      <button
        class="absolute top-10 right-1/2 translate-x-2 w-10 p-2 z-50 aspect-square bg-sky-400 rounded-full flex items-center justify-center text-white active:scale-75 hover:-rotate-90 duration-300 !cursor-pointer"
        onclick={(e) => {
          e.stopPropagation();
          e.preventDefault();
          reset();
        }}
      >
        <RotateCcw size="20" />
      </button>
    {/if}
    <div class="h-full flex flex-col overflow-y-auto scrollbar-hide">
      <DropzoneArea
        {files}
        addfiles={(files) => addFiles(files)}
        deletefile={(file) => deleteFile(file)}
        reorderfiles={(files) => reorderFiles(files)}
        {reset}
        dndcomplete={createGif}
      />
    </div>

    <div class="h-full flex flex-col gap-2 p-2 overflow-y-auto scrollbar-hide">
      <div class="text-xl font-mono text-center text-sky-500">export</div>
      <SettingsPanel
        {files}
        {timeValue}
        {bitDepthValue}
        {isConfigSaved}
        {isLoading}
        {isError}
        {generatedGif}
        {progressStatus}
        {progressPercent}
        {saveConfig}
        saveGif={save_gif}
      ></SettingsPanel>
    </div>
  </div>
</div>
