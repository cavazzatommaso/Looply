<script lang="ts">
  export let id: string;
  export let value: number;
  export let min: number = 0;
  export let max: number = 100;
  export let step: number = 1;
</script>

<div class="relative w-full h-full flex items-center">
  <input
    type="range"
    {id}
    bind:value
    {min}
    {max}
    {step}
    class="absolute
        left-1/2 top-1/2
        -translate-x-1/2 -translate-y-1/2
        transform -rotate-90
        origin-center
        w-[12rem]
        range
        range-sm
        text-sky-400
        appearance-none cursor-pointer [--range-thumb:#fff] [--range-fill:1]"
  />
</div>
