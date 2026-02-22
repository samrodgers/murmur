<script lang="ts">
  import { createReply } from "../lib/api";

  interface Props {
    parentId: string;
    authorName: string;
    onReplyCreated?: (reply: any) => void;
  }

  let { parentId, authorName, onReplyCreated }: Props = $props();

  let content = $state("");
  let isSubmitting = $state(false);

  async function handleSubmit() {
    const trimmed = content.trim();
    if (!trimmed || isSubmitting) return;

    isSubmitting = true;
    try {
      const reply = await createReply(trimmed, parentId);
      content = "";
      onReplyCreated?.(reply);
    } catch (e) {
      console.error("Failed to create reply:", e);
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

<div class="bg-white border border-gray-200 rounded-lg p-3">
  <textarea
    bind:value={content}
    onkeydown={handleKeydown}
    placeholder="Reply to {authorName}..."
    maxlength="500"
    rows="2"
    class="w-full resize-none text-sm text-gray-800 placeholder-gray-400 border-none outline-none bg-transparent"
    disabled={isSubmitting}
  ></textarea>

  <div class="flex items-center justify-end mt-1">
    <button
      onclick={handleSubmit}
      disabled={!content.trim() || isSubmitting}
      class="px-3 py-1 bg-orange-500 text-white text-xs font-medium rounded-lg
        hover:bg-orange-600 disabled:opacity-50 disabled:cursor-not-allowed
        transition-colors cursor-pointer"
    >
      {isSubmitting ? "..." : "Reply 📤"}
    </button>
  </div>
</div>
