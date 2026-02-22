<script lang="ts">
  interface Props {
    pubkey: string;
    onClose: () => void;
  }

  let { pubkey, onClose }: Props = $props();

  function copyToClipboard() {
    navigator.clipboard.writeText(pubkey);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
  onclick={onClose}
  onkeydown={(e) => e.key === 'Escape' && onClose()}
>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="bg-white rounded-xl p-6 max-w-sm w-full mx-4 shadow-xl"
    onclick={(e) => e.stopPropagation()}
  >
    <h3 class="text-lg font-semibold text-gray-900 mb-4">Your Murmur ID</h3>

    <div class="bg-gray-50 rounded-lg p-4 mb-4">
      <p class="text-xs font-mono text-gray-600 break-all select-all">
        {pubkey}
      </p>
    </div>

    <p class="text-sm text-gray-500 mb-4">
      Share this ID with friends so they can find you on the network.
    </p>

    <div class="flex gap-2">
      <button
        onclick={copyToClipboard}
        class="flex-1 px-4 py-2 bg-orange-500 text-white text-sm font-medium rounded-lg
          hover:bg-orange-600 transition-colors cursor-pointer"
      >
        Copy ID
      </button>
      <button
        onclick={onClose}
        class="px-4 py-2 bg-gray-100 text-gray-700 text-sm font-medium rounded-lg
          hover:bg-gray-200 transition-colors cursor-pointer"
      >
        Close
      </button>
    </div>
  </div>
</div>
