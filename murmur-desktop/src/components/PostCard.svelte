<script lang="ts">
  import type { Post } from "../lib/types";
  import { relativeTime } from "../lib/utils";
  import { store } from "../lib/stores.svelte";
  import LetterAvatar from "./LetterAvatar.svelte";
  import TemperatureBadge from "./TemperatureBadge.svelte";
  import EngagementBar from "./EngagementBar.svelte";

  interface Props {
    post: Post;
  }

  let { post }: Props = $props();

  function openThread() {
    store.navigate({ page: "thread", postId: post.id });
  }

  function openProfile(e: Event) {
    e.stopPropagation();
    store.navigate({ page: "profile", pubkey: post.author.pubkey });
  }

  function openReply() {
    store.navigate({ page: "thread", postId: post.id });
  }

  function stopProp(e: Event) {
    e.stopPropagation();
  }
</script>

<article
  class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow cursor-pointer"
  onclick={openThread}
  onkeydown={(e) => e.key === 'Enter' && openThread()}
  role="article"
  tabindex="0"
>
  <div class="flex items-start gap-3">
    <button onclick={openProfile} class="cursor-pointer">
      <LetterAvatar name={post.author.name} colour={post.author.avatar_colour} />
    </button>

    <div class="flex-1 min-w-0">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <button
            onclick={openProfile}
            class="font-semibold text-sm text-gray-900 hover:underline cursor-pointer"
          >
            {post.author.name}
          </button>
          <span class="text-xs text-gray-400">·</span>
          <span class="text-xs text-gray-400">{relativeTime(post.created_at)}</span>
        </div>
        <TemperatureBadge temperature={post.temperature} />
      </div>

      <p class="mt-1.5 text-sm text-gray-800 leading-relaxed whitespace-pre-wrap break-words">
        {post.content}
      </p>

      <div onclick={stopProp}>
        <EngagementBar
          postId={post.id}
          replyCount={post.reply_count}
          repostCount={post.repost_count}
          reactionCount={post.reaction_count}
          hasReacted={post.has_reacted}
          hasReposted={post.has_reposted}
          onReplyClick={openReply}
        />
      </div>
    </div>
  </div>
</article>
