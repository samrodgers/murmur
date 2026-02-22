<script lang="ts">
  import { store } from "../lib/stores";
</script>

<footer class="flex items-center justify-between px-4 py-2 border-t border-gray-200 bg-white text-xs text-gray-500">
  <div class="flex items-center gap-2">
    <span class="w-2 h-2 rounded-full {store.isOnline ? 'bg-green-500' : 'bg-red-400'}"></span>
    <span>
      {store.peerCount} peer{store.peerCount !== 1 ? 's' : ''} connected
    </span>
  </div>
  <span class="text-gray-400">Murmur v0.1.0</span>
</footer>
