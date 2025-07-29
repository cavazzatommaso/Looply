<script lang="ts">
  import { dndzone } from "svelte-dnd-action";
  import Dropzone from "svelte-file-dropzone";
  import type { UIFile } from "$lib/utils/fileUtils";
  import { RotateCcw, Trash } from "lucide-svelte";

  type DropzoneProps = {
    files: UIFile[];
    addfiles: (detail: File[]) => void;
    reorderfiles: (detail: UIFile[]) => void;
    dndcomplete: () => void;
    deletefile: (detail: string) => void;
    reset: () => void;
  };
  let {
    files,
    addfiles,
    reorderfiles,
    dndcomplete,
    deletefile,
    reset,
  }: DropzoneProps = $props();

  function handleFilesSelect(e: CustomEvent) {
    const { acceptedFiles } = e.detail;    
    addfiles(acceptedFiles);
  }

  function handleDndTrigger(e: CustomEvent) {
    const { detail } = e;
    reorderfiles(detail.items);

    if (detail.info.trigger === "droppedIntoZone") {
      dndcomplete();
    }
  }

  function deleteFile(fileId: string) {
    deletefile(fileId);
  }
</script>

<div class="p-2 grid grid-cols-1 grid-rows-[min-content_1fr] gap-2 flex-1">
  <div class="text-xl font-mono text-center text-sky-500">edit</div>
  <Dropzone on:drop={handleFilesSelect} containerClasses={"relative"} accept="image/*">
    <div
      use:dndzone={{ items: files, flipDurationMs: 300 }}
      onconsider={handleDndTrigger}
      onfinalize={handleDndTrigger}
      class="flex flex-wrap gap-2 flex-col items-center"
    >
    
      {#each files as file (file.id)}
        <div class={`relative p-2 group`}>
          {#if file}
            <span class="absolute top-1 -left-1 -translate-x-1/2 block text-black"
              >{files.indexOf(file)}</span
            >
            <button
              class="hidden z-100 absolute top-2 right-2 group-hover:flex w-5 aspect-square bg-red-500 translate-x-1/2 -translate-y-1/2 rounded-sm items-center justify-center text-white text-sm cursor-pointer"
              onclick={(e) => {
                e.stopPropagation();
                e.preventDefault();
                deleteFile(file.id);
              }}><Trash  size={12}/></button
            >
            <img
              src={URL.createObjectURL(file.file)}
              alt={file.file.name}
              class="relative object-fill rounded-md"
            />
          {/if}
        </div>
      {/each}
    </div>
  </Dropzone>
</div>
