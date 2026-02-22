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
    try {
      const exists = await hasIdentity();
      if (exists) {
        const profile = await getOwnProfile();
        store.setProfile(profile);
        store.navigate({ page: "timeline" });

        // Check network status; if offline, actively try to start it.
        const net = await getNetworkStatus();
        store.setOnline(net.online);
        store.setPeerCount(net.peer_count);

        if (!net.online) {
          try {
            const retry = await startNetwork();
            store.setOnline(retry.online);
            store.setPeerCount(retry.peer_count);
            if (retry.error) {
              store.setNetworkError(retry.error);
            }
          } catch (e) {
            store.setNetworkError(String(e));
          }
        }
      }
    } catch (e) {
      console.error("Startup check failed:", e);
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
