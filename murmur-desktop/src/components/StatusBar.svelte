<script lang="ts">
  import { store } from "../lib/stores.svelte";
  import { startNetwork } from "../lib/api";

  let isRetrying = $state(false);

  async function handleRetry() {
    isRetrying = true;
    try {
      const result = await startNetwork();
      store.setOnline(result.online);
      store.setPeerCount(result.peer_count);
      store.setNetworkError(result.error ?? null);
    } catch (e) {
      store.setNetworkError(String(e));
    } finally {
      isRetrying = false;
    }
  }
</script>

<footer class="border-t border-gray-200 bg-white text-xs text-gray-500">
  <div class="flex items-center justify-between px-4 py-2">
    <div class="flex items-center gap-2">
      <span class="w-2 h-2 rounded-full {store.isOnline ? 'bg-green-500' : 'bg-red-400'}"></span>
      {#if store.isOnline}
        <span>{store.peerCount} peer{store.peerCount !== 1 ? 's' : ''} connected</span>
      {:else}
        <span>Offline</span>
        <button
          onclick={handleRetry}
          disabled={isRetrying}
          class="px-1.5 py-0.5 bg-orange-500 text-white rounded
            hover:bg-orange-600 disabled:opacity-50 transition-colors cursor-pointer"
        >
          {isRetrying ? "..." : "Retry"}
        </button>
      {/if}
    </div>
    <span class="text-gray-400">Murmur v0.1.0</span>
  </div>
  {#if store.networkError && !store.isOnline}
    <div class="px-4 pb-2 text-red-600 truncate">
      {store.networkError}
    </div>
  {/if}
</footer>
