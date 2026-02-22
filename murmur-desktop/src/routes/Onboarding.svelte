<script lang="ts">
  import { createIdentity } from "../lib/api";
  import { store } from "../lib/stores.svelte";

  let name = $state("");
  let bio = $state("");
  let isCreating = $state(false);
  let error = $state("");

  async function handleGetStarted() {
    if (!name.trim()) {
      error = "Please enter a name";
      return;
    }

    isCreating = true;
    error = "";

    try {
      const profile = await createIdentity(name.trim(), bio.trim());
      store.setProfile(profile);
      store.navigate({ page: "timeline" });
    } catch (e) {
      error = `Something went wrong: ${e}`;
    } finally {
      isCreating = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && name.trim()) {
      handleGetStarted();
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-[#FAFAF8] p-6">
  <div class="max-w-md w-full text-center">
    <div class="mb-8">
      <span class="text-5xl">🔥</span>
      <h1 class="mt-4 text-3xl font-bold text-gray-900" style="font-family: 'Inter', serif;">
        Welcome to Murmur
      </h1>
      <p class="mt-3 text-gray-500 leading-relaxed">
        The social app where conversations spread like murmurs at a party.
      </p>
    </div>

    <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 text-left">
      <label class="block text-sm font-medium text-gray-700 mb-2">
        What should people call you?
      </label>
      <input
        bind:value={name}
        onkeydown={handleKeydown}
        type="text"
        placeholder="Your name"
        maxlength="50"
        class="w-full px-4 py-2.5 border border-gray-300 rounded-lg text-sm
          focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-transparent
          placeholder-gray-400"
        disabled={isCreating}
      />

      <label class="block text-sm font-medium text-gray-700 mt-4 mb-2">
        Optionally add a short bio:
      </label>
      <input
        bind:value={bio}
        type="text"
        placeholder="A few words about you..."
        maxlength="160"
        class="w-full px-4 py-2.5 border border-gray-300 rounded-lg text-sm
          focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-transparent
          placeholder-gray-400"
        disabled={isCreating}
      />

      {#if error}
        <p class="mt-3 text-sm text-red-500">{error}</p>
      {/if}

      <button
        onclick={handleGetStarted}
        disabled={!name.trim() || isCreating}
        class="w-full mt-6 px-4 py-3 bg-orange-500 text-white font-semibold rounded-lg
          hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed
          transition-colors cursor-pointer text-sm"
      >
        {isCreating ? "Setting up..." : "Get Started →"}
      </button>
    </div>

    <p class="mt-6 text-xs text-gray-400 leading-relaxed">
      Your identity is created on your device.<br />
      No email or password needed.
    </p>
  </div>
</div>
