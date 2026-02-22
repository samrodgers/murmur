<script lang="ts">
  import { store } from "../lib/stores";
  import LetterAvatar from "./LetterAvatar.svelte";

  function goToSettings() {
    store.navigate({ page: "settings" });
  }
</script>

<header class="flex items-center justify-between px-4 py-3 border-b border-gray-200 bg-white">
  <div class="flex items-center gap-2">
    <span class="text-xl">🔥</span>
    <h1 class="text-lg font-semibold text-gray-900" style="font-family: 'Inter', serif;">
      Murmur
    </h1>
  </div>

  <button
    onclick={goToSettings}
    class="flex items-center gap-2 px-3 py-1.5 rounded-lg hover:bg-gray-100 transition-colors cursor-pointer"
  >
    {#if store.profile}
      <LetterAvatar name={store.profile.name} colour={store.profile.avatar_colour} size="sm" />
      <span class="text-sm font-medium text-gray-700">{store.profile.name}</span>
    {/if}
    <span class="w-2 h-2 rounded-full {store.isOnline ? 'bg-green-500' : 'bg-gray-400'}"></span>
    <span class="text-xs text-gray-500">{store.isOnline ? 'Online' : 'Offline'}</span>
  </button>
</header>
