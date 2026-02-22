<script lang="ts">
  import { reactToPost, repostPost } from "../lib/api";

  interface Props {
    postId: string;
    replyCount: number;
    repostCount: number;
    reactionCount: number;
    hasReacted: boolean;
    hasReposted: boolean;
    onReplyClick?: () => void;
  }

  let {
    postId,
    replyCount,
    repostCount,
    reactionCount,
    hasReacted,
    hasReposted,
    onReplyClick,
  }: Props = $props();

  let localReactionCount = $state(0);
  let localHasReacted = $state(false);
  let localRepostCount = $state(0);
  let localHasReposted = $state(false);

  $effect(() => { localReactionCount = reactionCount; });
  $effect(() => { localHasReacted = hasReacted; });
  $effect(() => { localRepostCount = repostCount; });
  $effect(() => { localHasReposted = hasReposted; });

  async function handleReact() {
    if (localHasReacted) return;
    localReactionCount++;
    localHasReacted = true;
    try {
      await reactToPost(postId);
    } catch (e) {
      localReactionCount--;
      localHasReacted = false;
    }
  }

  async function handleRepost() {
    if (localHasReposted) return;
    localRepostCount++;
    localHasReposted = true;
    try {
      await repostPost(postId);
    } catch (e) {
      localRepostCount--;
      localHasReposted = false;
    }
  }
</script>

<div class="flex items-center gap-4 mt-2">
  <button
    onclick={onReplyClick}
    class="flex items-center gap-1 text-gray-500 hover:text-blue-500 transition-colors text-sm cursor-pointer"
  >
    💬 <span>{replyCount}</span>
  </button>

  <button
    onclick={handleRepost}
    class="flex items-center gap-1 text-sm transition-colors cursor-pointer
      {localHasReposted ? 'text-green-500' : 'text-gray-500 hover:text-green-500'}"
  >
    🔁 <span>{localRepostCount}</span>
  </button>

  <button
    onclick={handleReact}
    class="flex items-center gap-1 text-sm transition-all cursor-pointer
      {localHasReacted ? 'text-red-500 scale-110' : 'text-gray-500 hover:text-red-500'}"
  >
    ❤️ <span>{localReactionCount}</span>
  </button>
</div>
