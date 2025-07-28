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
      <div class="bg-gray-300 rounded-xl h-32 flex items-center justify-center text-red-500 font-bold">
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
    <div class="bg-gray-300 rounded-xl h-32 flex flex-col items-center justify-center text-gray-600 font-bold">
      {#if isLoading}
        <div class="w-full max-w-xs px-4">
          <!-- Progress Bar -->
          <div class="w-full bg-gray-500 rounded-full h-2 mb-3">
            <div 
              class="bg-sky-500 h-2 rounded-full transition-all duration-300 ease-out"
              style="width: {progressPercent}%"
            ></div>
          </div>
          
          <!-- Status Text -->
          <div class="text-sm text-center mb-2">
            {progressStatus || "Initializing..."}
          </div>
          
          <!-- Percentage -->
          <div class="text-xs text-center text-gray-500">
            {Math.round(progressPercent)}%
          </div>
        </div>
      {:else}
        preview here
      {/if}
    </div>
  {/if}
</div>