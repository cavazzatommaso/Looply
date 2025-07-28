<script lang="ts">
  export let generatedGif: string;
  export let hasFiles: boolean;
  export let isLoading: boolean;
  export let isError: boolean;
  export let progressStatus: string = "";
  export let progressPercent: number = 0;
</script>

<div class="h-full">
  {#if hasFiles && !isLoading}
    {#if isError}
      <div
        class="bg-gray-300 rounded-xl h-32 flex items-center justify-center text-red-500 font-bold"
      >
        Error generating the gif
      </div>
    {:else}
      <img
        src={generatedGif}
        alt="Generated Gif"
        class="min-h-32 object-contain border-3 border-gray-200 rounded-xl"
      />
    {/if}
  {:else}
    <div
      class="bg-gray-300 rounded-xl h-32 flex flex-col items-center justify-center text-gray-600 font-bold"
    >
      {#if isLoading}
        <div class="w-full max-w-xs px-4">
          <div class="flex gap-2 items-center justify-between">
            <progress class="progress progress-info w-full" value={progressPercent} max="100"></progress>
            <span class="text-xs">{Math.round(progressPercent)}%</span>
          </div>

          <div class="text-xs font-mono font-extralight">
            {progressStatus || "Initializing..."}
          </div>
        </div>
      {:else}
        preview here
      {/if}
    </div>
  {/if}
</div>
