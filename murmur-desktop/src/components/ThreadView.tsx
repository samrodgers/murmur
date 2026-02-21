import { createSignal, onMount, For, Show } from "solid-js";
import { getThread } from "../commands";
import type { MurmurEvent, Thread } from "../types";
import EventCard from "./EventCard";

interface ThreadViewProps {
  eventId: string;
  onBack: () => void;
  onReply: (event: MurmurEvent) => void;
  ownPubkey: string;
}

export default function ThreadView(props: ThreadViewProps) {
  const [thread, setThread] = createSignal<Thread | null>(null);
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    try {
      const t = await getThread(props.eventId);
      setThread(t);
    } catch (e) {
      console.error("Failed to load thread:", e);
    } finally {
      setLoading(false);
    }
  });

  return (
    <div class="thread-view">
      <div class="thread-header">
        <button class="back-btn" onClick={props.onBack}>
          &larr; Back
        </button>
        <h2>Thread</h2>
      </div>

      <Show when={!loading()} fallback={<div class="loading">Loading...</div>}>
        <Show
          when={thread()?.root}
          fallback={<div class="empty-state">Thread not found.</div>}
        >
          {(root) => (
            <>
              <div class="thread-root">
                <EventCard
                  event={root()}
                  onOpenThread={() => {}}
                  onReply={props.onReply}
                />
              </div>

              <Show when={thread()!.replies.length > 0}>
                <div class="thread-replies">
                  <div class="replies-header">
                    {thread()!.replies.length} repl
                    {thread()!.replies.length === 1 ? "y" : "ies"}
                  </div>
                  <For each={thread()!.replies}>
                    {(reply) => (
                      <EventCard
                        event={reply}
                        onOpenThread={() => {}}
                        onReply={props.onReply}
                        compact
                      />
                    )}
                  </For>
                </div>
              </Show>
            </>
          )}
        </Show>
      </Show>
    </div>
  );
}
