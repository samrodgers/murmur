<script lang="ts">
  import { onMount } from "svelte";
  import {
    getOwnProfile,
    updateProfile,
    getNetworkStatus,
    getStorageStats,
    setCacheLimit,
  } from "../lib/api";
  import { store } from "../lib/stores";
  import type { StorageStats, NetworkStatus } from "../lib/types";
  import LetterAvatar from "../components/LetterAvatar.svelte";
  import QRCodeModal from "../components/QRCodeModal.svelte";
  import StatusBar from "../components/StatusBar.svelte";

  let displayName = $state("");
  let bio = $state("");
  let networkStatus: NetworkStatus | null = $state(null);
  let storageStats: StorageStats | null = $state(null);
  let showQR = $state(false);
  let isSaving = $state(false);
  let saveMessage = $state("");

  onMount(() => {
    loadData();
  });

  async function loadData() {
    try {
      const profile = await getOwnProfile();
      displayName = profile.name;
      bio = profile.bio;
      store.setProfile(profile);

      networkStatus = await getNetworkStatus();
      storageStats = await getStorageStats();
    } catch (e) {
      console.error("Failed to load settings data:", e);
    }
  }

  function goBack() {
    store.navigate({ page: "timeline" });
  }

  async function handleSave() {
    isSaving = true;
    saveMessage = "";
    try {
      const profile = await updateProfile(displayName, bio);
      store.setProfile(profile);
      saveMessage = "Saved!";
      setTimeout(() => (saveMessage = ""), 2000);
    } catch (e) {
      saveMessage = `Error: ${e}`;
    } finally {
      isSaving = false;
    }
  }

  async function handleCacheLimitChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    const mb = parseInt(target.value);
    try {
      await setCacheLimit(mb);
      if (storageStats) {
        storageStats = { ...storageStats, cache_limit_mb: mb };
      }
    } catch (err) {
      console.error("Failed to set cache limit:", err);
    }
  }

  function copyPubkey() {
    if (store.profile) {
      navigator.clipboard.writeText(store.profile.pubkey);
    }
  }
</script>

<div class="flex flex-col h-screen bg-[#FAFAF8]">
  <header class="flex items-center gap-3 px-4 py-3 border-b border-gray-200 bg-white">
    <button onclick={goBack} class="text-gray-500 hover:text-gray-700 cursor-pointer text-sm">
      ← Back
    </button>
    <h2 class="font-semibold text-gray-900">Your Profile</h2>
  </header>

  <main class="flex-1 overflow-y-auto">
    <div class="max-w-lg mx-auto p-4 space-y-6">
      <!-- Profile edit -->
      <section class="bg-white border border-gray-200 rounded-lg p-5">
        <div class="flex items-center gap-3 mb-4">
          {#if store.profile}
            <LetterAvatar name={store.profile.name} colour={store.profile.avatar_colour} size="lg" />
          {/if}
        </div>

        <label class="block text-sm font-medium text-gray-700 mb-1">Display name</label>
        <input
          bind:value={displayName}
          type="text"
          maxlength="50"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm
            focus:outline-none focus:ring-2 focus:ring-orange-500 mb-3"
        />

        <label class="block text-sm font-medium text-gray-700 mb-1">Bio</label>
        <input
          bind:value={bio}
          type="text"
          maxlength="160"
          class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm
            focus:outline-none focus:ring-2 focus:ring-orange-500 mb-3"
        />

        <div class="flex items-center gap-2">
          <button
            onclick={handleSave}
            disabled={isSaving}
            class="px-4 py-2 bg-orange-500 text-white text-sm font-medium rounded-lg
              hover:bg-orange-600 disabled:opacity-50 transition-colors cursor-pointer"
          >
            {isSaving ? "Saving..." : "Save Changes"}
          </button>
          {#if saveMessage}
            <span class="text-sm text-green-600">{saveMessage}</span>
          {/if}
        </div>
      </section>

      <!-- Network -->
      <section class="bg-white border border-gray-200 rounded-lg p-5">
        <h3 class="font-semibold text-gray-900 mb-3">Network</h3>
        <div class="space-y-2 text-sm text-gray-600">
          <div class="flex justify-between">
            <span>Status</span>
            <span class="flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-full {store.isOnline ? 'bg-green-500' : 'bg-red-400'}"></span>
              {store.isOnline ? "Online" : "Offline"}
            </span>
          </div>
          <div class="flex justify-between">
            <span>Peers</span>
            <span class="font-medium text-gray-900">{store.peerCount} connected</span>
          </div>
          {#if storageStats}
            <div class="flex justify-between">
              <span>Your posts</span>
              <span class="font-medium text-gray-900">{storageStats.own_posts}</span>
            </div>
            <div class="flex justify-between">
              <span>Cached from others</span>
              <span class="font-medium text-gray-900">{storageStats.cached_posts}</span>
            </div>
            <div class="flex justify-between">
              <span>Storage used</span>
              <span class="font-medium text-gray-900">
                {storageStats.storage_used_mb.toFixed(1)} MB / {storageStats.cache_limit_mb} MB
              </span>
            </div>
          {/if}
        </div>
      </section>

      <!-- Storage Budget -->
      <section class="bg-white border border-gray-200 rounded-lg p-5">
        <h3 class="font-semibold text-gray-900 mb-3">Storage Budget</h3>
        <label class="block text-sm text-gray-600 mb-2">Cache limit</label>
        <select
          onchange={handleCacheLimitChange}
          class="px-3 py-2 border border-gray-300 rounded-lg text-sm
            focus:outline-none focus:ring-2 focus:ring-orange-500"
        >
          <option value="100">100 MB</option>
          <option value="250">250 MB</option>
          <option value="500" selected>500 MB</option>
          <option value="1000">1 GB</option>
          <option value="2000">2 GB</option>
        </select>
      </section>

      <!-- Identity -->
      <section class="bg-white border border-gray-200 rounded-lg p-5">
        <h3 class="font-semibold text-gray-900 mb-3">Identity</h3>
        {#if store.profile}
          <div class="bg-gray-50 rounded-lg p-3 mb-3">
            <p class="text-xs font-mono text-gray-500 break-all">
              {store.profile.pubkey}
            </p>
          </div>
          <div class="flex gap-2">
            <button
              onclick={copyPubkey}
              class="px-3 py-1.5 bg-gray-100 text-gray-700 text-sm rounded-lg
                hover:bg-gray-200 transition-colors cursor-pointer"
            >
              Copy ID
            </button>
            <button
              onclick={() => (showQR = true)}
              class="px-3 py-1.5 bg-gray-100 text-gray-700 text-sm rounded-lg
                hover:bg-gray-200 transition-colors cursor-pointer"
            >
              Show QR Code
            </button>
          </div>
        {/if}
      </section>
    </div>
  </main>

  <StatusBar />
</div>

{#if showQR && store.profile}
  <QRCodeModal pubkey={store.profile.pubkey} onClose={() => (showQR = false)} />
{/if}
