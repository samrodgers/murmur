<script lang="ts">
  import { onMount } from "svelte";
  import { getProfile, followUser, unfollowUser, blockUser } from "../lib/api";
  import { store } from "../lib/stores.svelte";
  import type { Profile } from "../lib/types";
  import LetterAvatar from "../components/LetterAvatar.svelte";
  import StatusBar from "../components/StatusBar.svelte";
  import { relativeTime } from "../lib/utils";

  interface Props {
    pubkey: string;
  }

  let { pubkey }: Props = $props();

  let profile: Profile | null = $state(null);
  let isLoading = $state(true);

  onMount(() => {
    loadProfile();
  });

  async function loadProfile() {
    isLoading = true;
    try {
      profile = await getProfile(pubkey);
    } catch (e) {
      console.error("Failed to load profile:", e);
    } finally {
      isLoading = false;
    }
  }

  function goBack() {
    store.navigate({ page: "timeline" });
  }

  async function handleFollow() {
    if (!profile) return;
    try {
      if (profile.is_following) {
        await unfollowUser(pubkey);
        profile = { ...profile, is_following: false };
      } else {
        await followUser(pubkey);
        profile = { ...profile, is_following: true };
      }
    } catch (e) {
      console.error("Follow/unfollow failed:", e);
    }
  }

  async function handleBlock() {
    if (!profile) return;
    try {
      await blockUser(pubkey);
      profile = { ...profile, is_blocked: true };
    } catch (e) {
      console.error("Block failed:", e);
    }
  }
</script>

<div class="flex flex-col h-screen bg-[#FAFAF8]">
  <header class="flex items-center gap-3 px-4 py-3 border-b border-gray-200 bg-white">
    <button onclick={goBack} class="text-gray-500 hover:text-gray-700 cursor-pointer text-sm">
      ← Back
    </button>
  </header>

  <main class="flex-1 overflow-y-auto">
    <div class="max-w-lg mx-auto p-4">
      {#if isLoading}
        <div class="text-center py-8 text-gray-400 text-sm">Loading profile...</div>
      {:else if profile}
        <div class="bg-white border border-gray-200 rounded-lg p-6">
          <div class="flex items-start gap-4">
            <LetterAvatar name={profile.name} colour={profile.avatar_colour} size="lg" />
            <div>
              <h2 class="text-xl font-bold text-gray-900">{profile.name}</h2>
              {#if profile.bio}
                <p class="text-sm text-gray-500 mt-1">"{profile.bio}"</p>
              {/if}
            </div>
          </div>

          <div class="mt-4 flex items-center gap-4 text-sm text-gray-500">
            <span>🌡️ Identity temp: <strong class="text-gray-700">{Math.round(profile.temperature)}</strong></span>
            <span>📝 {profile.post_count} posts</span>
            {#if profile.joined_at > 0}
              <span>🗓️ Joined {relativeTime(profile.joined_at)}</span>
            {/if}
          </div>

          <div class="mt-4 flex gap-2">
            <button
              onclick={handleFollow}
              class="px-4 py-2 text-sm font-medium rounded-lg transition-colors cursor-pointer
                {profile.is_following
                  ? 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                  : 'bg-orange-500 text-white hover:bg-orange-600'}"
            >
              {profile.is_following ? "Following" : "Follow"}
            </button>
            <button
              onclick={handleBlock}
              disabled={profile.is_blocked}
              class="px-4 py-2 text-sm font-medium rounded-lg bg-gray-100 text-gray-700
                hover:bg-red-100 hover:text-red-700 transition-colors cursor-pointer
                disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {profile.is_blocked ? "Blocked" : "Block"}
            </button>
          </div>
        </div>

        <div class="mt-6">
          <div class="flex items-center gap-2 mb-4">
            <div class="h-px flex-1 bg-gray-200"></div>
            <span class="text-xs text-gray-400 font-medium">Recent posts</span>
            <div class="h-px flex-1 bg-gray-200"></div>
          </div>
          <p class="text-center text-sm text-gray-400 py-4">
            Posts will appear here when this user is online.
          </p>
        </div>

        <div class="mt-6 bg-gray-50 rounded-lg p-3">
          <p class="text-xs text-gray-400 font-mono break-all">
            ID: {profile.pubkey}
          </p>
        </div>
      {/if}
    </div>
  </main>

  <StatusBar />
</div>
