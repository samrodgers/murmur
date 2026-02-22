<script lang="ts">
  import { avatarLetter } from "../lib/utils";

  interface Props {
    name: string;
    colour: string;
    size?: "sm" | "md" | "lg";
  }

  let { name, colour, size = "md" }: Props = $props();

  const sizeClasses = {
    sm: "w-7 h-7 text-xs",
    md: "w-10 h-10 text-sm",
    lg: "w-14 h-14 text-lg",
  };
</script>

<div
  class="rounded-full flex items-center justify-center font-semibold text-white shrink-0 {sizeClasses[size]}"
  style="background-color: {colour}"
>
  {avatarLetter(name)}
</div>
