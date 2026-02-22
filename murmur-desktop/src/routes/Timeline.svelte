<script lang="ts">
  import { onMount } from "svelte";
  import { getFollowingFeed, getDiscoverFeed, getNewVoicesFeed } from "../lib/api";
  import { onNewPost, onPeerChanged } from "../lib/events";
  import { store } from "../lib/stores";
  import type { FeedTab } from "../lib/types";
  import TopBar from "../components/TopBar.svelte";
  import FeedTabs from "../components/FeedTabs.svelte";
  import ComposeBox from "../components/ComposeBox.svelte";
  import PostCard from "../components/PostCard.svelte";
  import StatusBar from "../components/StatusBar.svelte";

  let isLoading = $state(true);

  onMount(() => {
    loadFeed(store.currentTab);

    // Subscribe to real-time events
    const unlistenPost = onNewPost((post) => {
      store.addPost(post);
    });

    const unlistenPeer = onPeerChanged((payload) => {
      store.setPeerCount(payload.connected_count);
    });

    return () => {
      unlistenPost.then((fn) => fn());
      unlistenPeer.then((fn) => fn());
    };
  });

  async function loadFeed(tab: FeedTab) {
    isLoading = true;
    try {
      let posts;
      switch (tab) {
        case "following":
          posts = await getFollowingFeed(50);
          break;
        case "discover":
          posts = await getDiscoverFeed(50);
          break;
        case "new-voices":
          posts = await getNewVoicesFeed(50);
          break;
      }
      store.setTimeline(posts);
    } catch (e) {
      console.error("Failed to load feed:", e);
    } finally {
      isLoading = false;
    }
  }

  function handleTabChange(tab: FeedTab) {
    store.currentTab = tab;
    loadFeed(tab);
  }
</script>

<div class="flex flex-col h-screen bg-[#FAFAF8]">
  <TopBar />
  <FeedTabs activeTab={store.currentTab} onTabChange={handleTabChange} />

  <main class="flex-1 overflow-y-auto">
    <div class="max-w-lg mx-auto p-4 space-y-4">
      <ComposeBox />

      {#if isLoading}
        <div class="text-center py-8 text-gray-400 text-sm">
          Loading...
        </div>
      {:else if store.timeline.length === 0}
        <div class="text-center py-12">
          <span class="text-4xl">👋</span>
          <h3 class="mt-4 text-lg font-semibold text-gray-700">
            {store.peerCount === 0 ? "You're the first one here!" : "No posts yet"}
          </h3>
          <p class="mt-2 text-sm text-gray-500 leading-relaxed max-w-xs mx-auto">
            {#if store.peerCount === 0}
              Murmur finds other users on your network automatically.
              Ask a friend to install the app and they'll appear here in seconds.
            {:else}
              Start the conversation! Write something above and it'll spread to your peers.
            {/if}
          </p>
          <p class="mt-4 text-xs text-gray-400">
            Meanwhile, you can start posting — your posts will spread as soon as someone connects.
          </p>
        </div>
      {:else}
        {#each store.timeline as post (post.id)}
          <PostCard {post} />
        {/each}
      {/if}
    </div>
  </main>

  <StatusBar />
</div>
