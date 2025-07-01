<script lang="ts">
  import { FFmpeg } from "@ffmpeg/ffmpeg";
  import { fetchFile, toBlobURL } from "@ffmpeg/util";
  import { onMount } from "svelte";
  import { dndzone } from "svelte-dnd-action";
  import { save } from "@tauri-apps/plugin-dialog";
  import { invoke } from "@tauri-apps/api/core";
  import Slider from "$lib/components/Slider.svelte";
  import arrow from "$lib/assets/arrow.png";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { listen } from "@tauri-apps/api/event";
  import { getFilesFromPaths } from "$lib/utils";
  import Dropzone from "svelte-file-dropzone";
  import { writable, type Writable } from "svelte/store";

  let ffmpeg: FFmpeg;
  let isLoading = false;
  let isReady = false;

  type UIFile = {
    id: string;
    file: File;
  };

  let files: UIFile[] = [];
  let generatedGif: string = "";
  let outputUrl: string | null = null;
  let timeValue: Writable<number> = writable(0.5);
  let bitDepthValue: Writable<number> = writable(0.5);
  const KEY_time = "gif_time";
  const KEY_bit = "gif_bit";
  let isOver: boolean = false;
  type DragDropPayload = {
    paths: string[];
    position: { x: number; y: number };
  };
  const baseURL = "https://unpkg.com/@ffmpeg/core@0.12.6/dist/esm";

  listen<DragDropPayload>("tauri://drag-drop", async (event) => {
    const { paths, position } = event.payload;
    const rawFiles = await getFilesFromPaths(paths);

    const images = rawFiles.filter((file) => file.type.startsWith("image/"));
    addFiles(images);
  });

  let timeValueTimer: number | undefined;

  const debounce = () => {
    clearTimeout(timeValueTimer);
    timeValueTimer = setTimeout(() => {
      createGif();
    }, 1000);
  };

  timeValue.subscribe(debounce);

  onMount(async () => {
    isLoading = true;

    const storedTime = localStorage.getItem(KEY_time) || 0.5;
    const storedBit = localStorage.getItem(KEY_bit) || 256;

    timeValue.set(Number(storedTime));
    bitDepthValue.set(Number(storedBit));

    const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        isOver = true;
      } else if (event.payload.type === "drop") {
        isOver = false;
      } else {
        isOver = false;
      }
    });

    ffmpeg = new FFmpeg();
    await loadffmpeg();
    isLoading = false;
    isReady = true;
  });

  async function loadffmpeg() {
    await ffmpeg.load({
      coreURL: await toBlobURL(`${baseURL}/ffmpeg-core.js`, "text/javascript"),
      wasmURL: await toBlobURL(
        `${baseURL}/ffmpeg-core.wasm`,
        "application/wasm",
      ),
    });
  }

  function saveConfig() {
    localStorage.setItem(KEY_time, $timeValue.toString());
    localStorage.setItem(KEY_bit, $bitDepthValue.toString());
  }

  function addFiles(newFiles: File[]) {
    const factoredFiles: UIFile[] = newFiles.map((file: any) => ({
      id: crypto.randomUUID(),
      file,
    }));

    files = [...files, ...factoredFiles];
    createGif();
  }

  function deleteFile(fileId: string) {
    files = files.filter((file) => file.id !== fileId);
    console.log(files);

    createGif();
  }

  function handleFilesSelect(e: CustomEvent) {
    const { acceptedFiles, fileRejections } = e.detail;
    addFiles(acceptedFiles);
  }

  function handleDndTrigger({ detail }: CustomEvent) {
    files = detail.items;

    if (detail.info.trigger === "droppedIntoZone") createGif();
  }

  async function createGif() {
    if (!isReady || files.length === 0) {
      // alert("FFmpeg not ready or no files.");
      return;
    }

    isLoading = true;

    ffmpeg.terminate();
    await loadffmpeg();

    const tempDir = "/tmp";

    for (const [i, file] of files.entries()) {
      const ext = file.file.name.split(".").pop() || "jpeg";
      const inputName = `${tempDir}/img${i}.${ext}`;
      await ffmpeg.writeFile(inputName, await fetchFile(file.file));

      const pngName = `${tempDir}/img${i}.png`;
      await ffmpeg.exec(["-i", inputName, pngName]);
    }

    const fps = (1 / $timeValue).toFixed(2);

    try {
      await ffmpeg.exec([
        "-framerate",
        `${fps}`,
        "-start_number",
        "0",
        "-i",
        `${tempDir}/img%d.png`,
        "-loop",
        "0",
        "output.gif",
      ]);

      const data = await ffmpeg.readFile("output.gif");
      generatedGif = URL.createObjectURL(
        new Blob([data], { type: "image/gif" }),
      );
    } catch (error) {
      generatedGif = "";
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
    <div
      class="p-2 grid grid-cols-1 grid-rows-[min-content_1fr] gap-2 overflow-x-hidden"
    >
      <div class="text-xl font-mono text-center text-sky-500">edit</div>
      <Dropzone on:drop={handleFilesSelect}>
        <div
          use:dndzone={{ items: files, flipDurationMs: 300 }}
          on:consider={handleDndTrigger}
          on:finalize={handleDndTrigger}
          class="flex flex-wrap gap-2 flex-col items-center"
        >
          {#each files as file (file.id)}
            <div class={`relative p-2 group`}>
              {#if file}
                <span class="absolute top-0 -left-2 block text-black"
                  >{files.indexOf(file)}</span
                >
                <button
                  class="hidden z-100 absolute top-2 right-2 group-hover:flex w-4 aspect-square bg-gray-200 translate-x-1/2 -translate-y-1/2 rounded-xl items-center justify-center text-sky-500 text-sm cursor-pointer hover:scale-110 active:scale-50 duration-300"
                  on:click|stopPropagation|preventDefault={() =>
                    deleteFile(file.id)}>x</button
                >
                <img
                  src={URL.createObjectURL(file.file)}
                  alt={file.file.name}
                  class="relative object-fill rounded-md group-hover:mask-r-from-50% duration-300"
                />
              {/if}
            </div>
          {/each}
        </div>
      </Dropzone>
    </div>
    <div class="h-full flex flex-col gap-2 p-2">
      <div class="text-xl font-mono text-center text-sky-500">export</div>
      <div
        class="h-full grid grid-cols-1 grid-rows-[min-content_2fr_auto_1fr] gap-2 bg-white p-1 rounded-xl"
      >
        <div class=" ">
          {#if files.length}
            <img
              src={generatedGif}
              alt="Generated Gif"
              class="min-h-32 object-contain border-3 border-gray-200 rounded-xl"
            />
          {:else}
            <div
              class="bg-gray-300 rounded-xl h-32 flex items-center justify-center text-gray-600 font-bold"
            >
              preview here
            </div>
          {/if}
        </div>

        <div class="grid grid-cols-[1fr_auto_1fr] relative">
          <div class="flex flex-col justify-around">
            <div class="font-mono text-center text-sm">time</div>
            <div class="flex-1">
              <Slider
                id="time-slider"
                bind:value={timeValue}
                min={0.25}
                max={5}
                step={0.25}
              />
            </div>
          </div>
          <div></div>
          <div class="flex flex-col">
            <div class="font-mono text-center text-sm">bit depth</div>
            <div class="flex-1">
              <Slider
                id="bit-slider"
                bind:value={bitDepthValue}
                min={0}
                max={256}
                step={16}
              />
            </div>
          </div>
        </div>
        <div
          class="grid grid-cols-[1fr_10px_1fr] border-3 border-gray-200 rounded-xl relative text-lg font-mono items-center p-1"
        >
          <button
            class="absolute top-0 right-0 w-5 aspect-square bg-sky-400 translate-x-1/2 -translate-y-1/2 rounded-full flex items-center justify-center text-white text-sm hover:scale-125 active:scale-50 duration-300"
            on:click|preventDefault={saveConfig}
          >
            +
          </button>
          <div class="text-center">{$timeValue}</div>
          <div class="divider divider-horizontal m-0"></div>
          <div class="text-center">{$bitDepthValue}</div>
        </div>
        <button
          class="w-full h-full bg-sky-400 rounded-xl flex items-center justify-center cursor-pointer"
          on:click={save_gif}
        >
          <img src={arrow} alt="Save gif button" srcset="" />
        </button>
      </div>
    </div>
  </div>
</div>
