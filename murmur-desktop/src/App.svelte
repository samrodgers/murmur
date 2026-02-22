<script lang="ts">
  import { onMount } from "svelte";
  import { hasIdentity, getOwnProfile, getNetworkStatus, startNetwork } from "./lib/api";
  import { store } from "./lib/stores.svelte";
  import Onboarding from "./routes/Onboarding.svelte";
  import Timeline from "./routes/Timeline.svelte";
  import ThreadView from "./routes/ThreadView.svelte";
  import ProfileView from "./routes/ProfileView.svelte";
  import Settings from "./routes/Settings.svelte";

  let ready = $state(false);

  onMount(async () => {
    // 1. Check identity and load profile
    try {
      const exists = await hasIdentity();
      if (exists) {
        const profile = await getOwnProfile();
        store.setProfile(profile);
        store.navigate({ page: "timeline" });
      }
    } catch (e) {
      console.error("Startup check failed:", e);
    }

    // 2. Always try to bring the network up (separate from identity check)
    if (store.profile) {
      // First, try startNetwork directly — it's a no-op if already running
      try {
        const result = await startNetwork();
        store.setOnline(result.online);
        store.setPeerCount(result.peer_count);
        if (result.error) {
          store.setNetworkError(result.error);
        }
      } catch (e) {
        store.setNetworkError(`Network start failed: ${e}`);
      }
    }

    ready = true;
  });
</script>

{#if !ready}
  <div class="min-h-screen flex items-center justify-center bg-[#FAFAF8]">
    <div class="text-center">
      <span class="text-4xl">🔥</span>
      <p class="mt-3 text-sm text-gray-400">Starting Murmur...</p>
    </div>
  </div>
{:else if store.route.page === "onboarding"}
  <Onboarding />
{:else if store.route.page === "timeline"}
  <Timeline />
{:else if store.route.page === "thread"}
  <ThreadView postId={store.route.postId} />
{:else if store.route.page === "profile"}
  <ProfileView pubkey={store.route.pubkey} />
{:else if store.route.page === "settings"}
  <Settings />
{/if}
