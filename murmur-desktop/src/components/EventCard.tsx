import type { MurmurEvent } from "../types";

interface EventCardProps {
  event: MurmurEvent;
  onOpenThread: (id: string) => void;
  onReply: (event: MurmurEvent) => void;
  compact?: boolean;
}

export default function EventCard(props: EventCardProps) {
  const kindBadge = () => {
    if (props.event.kind_label === "reply") return "reply";
    return "";
  };

  return (
    <div
      class={`event-card ${props.event.is_own ? "own" : ""} ${props.compact ? "compact" : ""}`}
    >
      <div class="event-header">
        <span class={`event-author ${props.event.is_own ? "you" : ""}`}>
          {props.event.is_own ? "you" : props.event.pubkey_short}
        </span>
        {kindBadge() && <span class="event-badge">{kindBadge()}</span>}
        <span class="event-time">{props.event.time_ago}</span>
      </div>

      <div class="event-content">{props.event.content}</div>

      <div class="event-actions">
        <button
          class="action-btn"
          onClick={() => props.onOpenThread(props.event.id)}
          title="View thread"
        >
          thread
        </button>
        <button
          class="action-btn"
          onClick={() => props.onReply(props.event)}
          title="Reply"
        >
          reply
        </button>
        <span class="event-id" title={props.event.id}>
          {props.event.id.slice(0, 12)}...
        </span>
      </div>
    </div>
  );
}
