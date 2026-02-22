<script lang="ts">
  import { createPost } from "../lib/api";
  import { store } from "../lib/stores.svelte";

  let content = $state("");
  let isSubmitting = $state(false);
  let charsLeft = $derived(500 - content.length);

  async function handleSubmit() {
    const trimmed = content.trim();
    if (!trimmed || isSubmitting) return;

    isSubmitting = true;
    try {
      const post = await createPost(trimmed);
      store.addPost(post);
      content = "";
    } catch (e) {
      console.error("Failed to create post:", e);
    } finally {
      isSubmitting = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      handleSubmit();
    }
  }
</script>

<div class="bg-white border border-gray-200 rounded-lg p-4">
  <textarea
    bind:value={content}
    onkeydown={handleKeydown}
    placeholder="What's on your mind?"
    maxlength="500"
    rows="3"
    class="w-full resize-none text-sm text-gray-800 placeholder-gray-400 border-none outline-none bg-transparent"
    disabled={isSubmitting}
  ></textarea>

  <div class="flex items-center justify-between mt-2">
    <span class="text-xs {charsLeft < 50 ? 'text-orange-500' : 'text-gray-400'}">
      {charsLeft}/500
    </span>
    <button
      onclick={handleSubmit}
      disabled={!content.trim() || isSubmitting || charsLeft < 0}
      class="px-4 py-1.5 bg-orange-500 text-white text-sm font-medium rounded-lg
        hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed
        transition-colors cursor-pointer"
    >
      {isSubmitting ? "Posting..." : "Post 📤"}
    </button>
  </div>
</div>
