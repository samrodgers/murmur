<script lang="ts">
  import type { FeedTab } from "../lib/types";

  interface Props {
    activeTab: FeedTab;
    onTabChange: (tab: FeedTab) => void;
  }

  let { activeTab, onTabChange }: Props = $props();

  const tabs: { id: FeedTab; label: string }[] = [
    { id: "following", label: "Following" },
    { id: "discover", label: "Discover" },
    { id: "new-voices", label: "New Voices" },
  ];
</script>

<div class="flex border-b border-gray-200 bg-white">
  {#each tabs as tab}
    <button
      onclick={() => onTabChange(tab.id)}
      class="flex-1 py-2.5 text-sm font-medium text-center transition-colors cursor-pointer
        {activeTab === tab.id
          ? 'text-orange-600 border-b-2 border-orange-500'
          : 'text-gray-500 hover:text-gray-700'}"
    >
      {tab.label}
    </button>
  {/each}
</div>
