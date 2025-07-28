<script lang="ts">
  import Slider from "$lib/components/Slider.svelte";
  import { type Writable } from "svelte/store";
  import arrow from "$lib/assets/arrow.png";
  import GifDisplay from "./GifDisplay.svelte";
  import type { UIFile } from "$lib/utils/fileUtils";

  type SettingsPanelProps = {
    files: UIFile[];
    timeValue: Writable<number>;
    bitDepthValue: Writable<number>;
    isConfigSaved: boolean;
    isLoading: boolean;
    isError: boolean;
    generatedGif: string;
    progressStatus: string;
    progressPercent: number;
    saveConfig: () => void;
    saveGif: () => void;
  };
  let {
    files,
    timeValue,
    bitDepthValue,
    isConfigSaved,
    isLoading,
    isError,
    generatedGif,
    progressStatus,
    progressPercent,
    saveConfig,
    saveGif,
  }: SettingsPanelProps = $props();
</script>

<div
  class="h-full grid grid-cols-1 grid-rows-[min-content_2fr_auto_1fr] gap-2 bg-white p-1 rounded-xl"
>
  <GifDisplay
    {generatedGif}
    hasFiles={files.length > 0}
    {isLoading}
    {isError}
    {progressStatus}
    {progressPercent}
  />

  <div class="grid grid-cols-[1fr_auto_1fr] relative">
    <div class="flex flex-col justify-around">
      <div class="font-mono text-center text-sm">time</div>
      <div class="flex-1">
        <Slider
          id="time-slider"
          bind:value={timeValue}
          min={0.1}
          max={5}
          step={0.1}
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
          step={[4, 8, 16, 32, 64, 128, 256]}
        />
      </div>
    </div>
  </div>
  <div
    class="grid grid-cols-[1fr_10px_1fr] border-3 border-gray-200 rounded-xl relative text-lg font-mono items-center p-1"
  >
    <button
      class="absolute top-0 right-0 w-5 aspect-square bg-sky-400 translate-x-1/2 -translate-y-1/2 rounded-full flex items-center justify-center text-white text-sm hover:scale-125 active:scale-50 duration-300"
      onclick={(e) => {
        e.preventDefault();
        saveConfig();
      }}
    >
      +
    </button>
    <div class="text-center">{$timeValue}</div>
    <div class="divider divider-horizontal m-0"></div>
    <div class="text-center">{$bitDepthValue}</div>
  </div>
  <button
    class="w-full h-full bg-sky-400 rounded-xl flex items-center justify-center cursor-pointer"
    onclick={saveGif}
    disabled={isLoading || !generatedGif}
  >
    <img src={arrow} alt="Save gif button" srcset="" />
  </button>
</div>

{#if isConfigSaved}
  <div class="toast toast-top toast-end">
    <div class="alert text-white bg-sky-400 border-sky-400">
      <span>Config saved!</span>
    </div>
  </div>
{/if}
