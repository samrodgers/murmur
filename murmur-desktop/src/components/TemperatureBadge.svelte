<script lang="ts">
  import { getTempLevel, getTempBgClass } from "../lib/temperature";

  interface Props {
    temperature: number;
  }

  let { temperature }: Props = $props();

  let level = $derived(getTempLevel(temperature));
  let bgClass = $derived(getTempBgClass(temperature));
</script>

<span
  class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium {bgClass}
    {level === 'fire' ? 'animate-pulse' : ''}"
>
  🌡️ {Math.round(temperature)}
</span>
