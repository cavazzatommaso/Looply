<script lang="ts">
  export let generatedGif: string;
  export let hasFiles: boolean;
  export let isLoading: boolean;
  export let isError: boolean;
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
      {isError}
    {/if}
  {:else}
    <div
      class="bg-gray-300 rounded-xl h-32 flex items-center justify-center text-gray-600 font-bold"
    >
      {#if isLoading}
        <span class="loading loading-dots bg-gray-600"></span>
      {:else}
        preview here
      {/if}
    </div>
  {/if}
</div>
