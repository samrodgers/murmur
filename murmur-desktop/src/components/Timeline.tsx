import { For, Show } from "solid-js";
import type { MurmurEvent } from "../types";
import EventCard from "./EventCard";

interface TimelineProps {
  events: MurmurEvent[];
  onOpenThread: (id: string) => void;
  onReply: (event: MurmurEvent) => void;
}

export default function Timeline(props: TimelineProps) {
  return (
    <div class="timeline">
      <Show
        when={props.events.length > 0}
        fallback={
          <div class="empty-state">
            <div class="empty-icon">~</div>
            <p>No posts yet.</p>
            <p class="empty-hint">
              Write the first post, or wait for peers to connect.
            </p>
          </div>
        }
      >
        <For each={props.events}>
          {(event) => (
            <EventCard
              event={event}
              onOpenThread={props.onOpenThread}
              onReply={props.onReply}
            />
          )}
        </For>
      </Show>
    </div>
  );
}
