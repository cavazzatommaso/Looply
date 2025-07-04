<script lang="ts">
  import { writable, type Writable, get } from "svelte/store";
  import { onDestroy } from "svelte";

  export let id: string;
  export let value: Writable<number>;
  export let min: number = 0;
  export let max: number = 100;
  export let step: number | number[] = 1;

  const internalSliderBindValue = writable(0);

  let unsubscribeExternalValue: () => void;
  let unsubscribeInternalBindValue: () => void;

  $: {
    if (unsubscribeExternalValue) unsubscribeExternalValue();
    if (unsubscribeInternalBindValue) unsubscribeInternalBindValue();

    if (Array.isArray(step)) {
      const currentExternalValue = get(value);
      const initialIndex = step.indexOf(currentExternalValue);
      internalSliderBindValue.set(initialIndex >= 0 ? initialIndex : 0);

      unsubscribeInternalBindValue = internalSliderBindValue.subscribe(
        (index) => {
          if (Array.isArray(step) && index >= 0 && index < step.length) {
            const newValue = step[index];
            if (get(value) !== newValue) {
              value.set(newValue);
            }
          }
        },
      );

      unsubscribeExternalValue = value.subscribe((val) => {
        if (Array.isArray(step)) {
          const newIndex = step.indexOf(val);
          if (get(internalSliderBindValue) !== newIndex) {
            internalSliderBindValue.set(newIndex >= 0 ? newIndex : 0);
          }
        }
      });
    } else {
      internalSliderBindValue.set(get(value));
      unsubscribeInternalBindValue = internalSliderBindValue.subscribe(
        (val) => {
          if (!Array.isArray(step) && get(value) !== val) {
            value.set(val);
          }
        },
      );

      unsubscribeExternalValue = value.subscribe((val) => {
        if (!Array.isArray(step) && get(internalSliderBindValue) !== val) {
          internalSliderBindValue.set(val);
        }
      });
    }
  }

  onDestroy(() => {
    if (unsubscribeExternalValue) unsubscribeExternalValue();
    if (unsubscribeInternalBindValue) unsubscribeInternalBindValue();
  });
</script>

<div class="relative w-full h-full flex items-center">
  <input
    type="range"
    {id}
    bind:value={$internalSliderBindValue}
    min={Array.isArray(step) ? 0 : min}
    max={Array.isArray(step) ? step.length - 1 : max}
    step={Array.isArray(step) ? 1 : step}
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
