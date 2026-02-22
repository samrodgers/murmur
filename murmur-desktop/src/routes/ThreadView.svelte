<script lang="ts">
  import { onMount } from "svelte";
  import { getThread } from "../lib/api";
  import { store } from "../lib/stores.svelte";
  import { relativeTime } from "../lib/utils";
  import type { Post, Thread } from "../lib/types";
  import LetterAvatar from "../components/LetterAvatar.svelte";
  import TemperatureBadge from "../components/TemperatureBadge.svelte";
  import EngagementBar from "../components/EngagementBar.svelte";
  import ReplyBox from "../components/ReplyBox.svelte";
  import StatusBar from "../components/StatusBar.svelte";

  interface Props {
    postId: string;
  }

  let { postId }: Props = $props();

  let thread: Thread | null = $state(null);
  let isLoading = $state(true);
  let error = $state("");

  onMount(() => {
    loadThread();
  });

  async function loadThread() {
    isLoading = true;
    error = "";
    try {
      thread = await getThread(postId);
    } catch (e) {
      error = `Failed to load thread: ${e}`;
    } finally {
      isLoading = false;
    }
  }

  function goBack() {
    store.navigate({ page: "timeline" });
  }

  function openProfile(pubkey: string) {
    store.navigate({ page: "profile", pubkey });
  }

  function handleReplyCreated(reply: Post) {
    if (thread) {
      thread = { ...thread, replies: [...thread.replies, reply] };
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
        <div class="text-center py-8 text-gray-400 text-sm">Loading thread...</div>
      {:else if error}
        <div class="text-center py-8 text-red-500 text-sm">{error}</div>
      {:else if thread}
        <!-- Root post -->
        <article class="bg-white border border-gray-200 rounded-lg p-5">
          <div class="flex items-start gap-3">
            <button onclick={() => openProfile(thread!.root.author.pubkey)} class="cursor-pointer">
              <LetterAvatar
                name={thread.root.author.name}
                colour={thread.root.author.avatar_colour}
                size="lg"
              />
            </button>
            <div class="flex-1">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <button
                    onclick={() => openProfile(thread!.root.author.pubkey)}
                    class="font-semibold text-gray-900 hover:underline cursor-pointer"
                  >
                    {thread.root.author.name}
                  </button>
                  <span class="text-xs text-gray-400">· {relativeTime(thread.root.created_at)}</span>
                </div>
                <TemperatureBadge temperature={thread.root.temperature} />
              </div>
              <p class="mt-3 text-gray-800 leading-relaxed whitespace-pre-wrap break-words">
                {thread.root.content}
              </p>
              <EngagementBar
                postId={thread.root.id}
                replyCount={thread.root.reply_count}
                repostCount={thread.root.repost_count}
                reactionCount={thread.root.reaction_count}
                hasReacted={thread.root.has_reacted}
                hasReposted={thread.root.has_reposted}
              />
            </div>
          </div>
        </article>

        <!-- Reply box -->
        <div class="mt-4">
          <ReplyBox
            parentId={thread.root.id}
            authorName={thread.root.author.name}
            onReplyCreated={handleReplyCreated}
          />
        </div>

        <!-- Replies -->
        {#if thread.replies.length > 0}
          <div class="mt-6">
            <div class="flex items-center gap-2 mb-4">
              <div class="h-px flex-1 bg-gray-200"></div>
              <span class="text-xs text-gray-400 font-medium">Replies</span>
              <div class="h-px flex-1 bg-gray-200"></div>
            </div>

            <div class="space-y-3">
              {#each thread.replies as reply (reply.id)}
                <article class="flex gap-3 pl-4 border-l-2 border-orange-200">
                  <button onclick={() => openProfile(reply.author.pubkey)} class="cursor-pointer">
                    <LetterAvatar
                      name={reply.author.name}
                      colour={reply.author.avatar_colour}
                      size="sm"
                    />
                  </button>
                  <div class="flex-1">
                    <div class="flex items-center gap-2">
                      <button
                        onclick={() => openProfile(reply.author.pubkey)}
                        class="font-semibold text-sm text-gray-900 hover:underline cursor-pointer"
                      >
                        {reply.author.name}
                      </button>
                      <span class="text-xs text-gray-400">· {relativeTime(reply.created_at)}</span>
                    </div>
                    <p class="mt-1 text-sm text-gray-700 whitespace-pre-wrap break-words">
                      {reply.content}
                    </p>
                    <EngagementBar
                      postId={reply.id}
                      replyCount={reply.reply_count}
                      repostCount={reply.repost_count}
                      reactionCount={reply.reaction_count}
                      hasReacted={reply.has_reacted}
                      hasReposted={reply.has_reposted}
                    />
                  </div>
                </article>
              {/each}
            </div>
          </div>
        {/if}
      {/if}
    </div>
  </main>

  <StatusBar />
</div>
