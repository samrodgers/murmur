import { Show, For } from "solid-js";
import type { Identity, Stats, MurmurEvent } from "../types";

interface ProfileViewProps {
  identity: Identity | null;
  stats: Stats;
  events: MurmurEvent[];
}

export default function ProfileView(props: ProfileViewProps) {
  const ownEvents = () => props.events.filter((e) => e.is_own);

  return (
    <div class="profile-view">
      <div class="profile-card">
        <div class="profile-avatar">
          <div class="avatar-placeholder">
            {props.identity?.pubkey_short.slice(0, 2) ?? "??"}
          </div>
        </div>

        <div class="profile-info">
          <h2 class="profile-name">{props.identity?.pubkey_short ?? "Loading..."}</h2>
          <p class="profile-pubkey" title={props.identity?.pubkey ?? ""}>
            {props.identity?.pubkey ?? ""}
          </p>
        </div>

        <div class="profile-stats">
          <div class="stat">
            <span class="stat-value">{props.stats.own_events}</span>
            <span class="stat-label">Posts</span>
          </div>
          <div class="stat">
            <span class="stat-value">{props.stats.total_events}</span>
            <span class="stat-label">Total Events</span>
          </div>
          <div class="stat">
            <span class="stat-value">{props.stats.cached_events}</span>
            <span class="stat-label">Cached</span>
          </div>
          <div class="stat">
            <span class="stat-value">{props.stats.peer_count}</span>
            <span class="stat-label">Peers</span>
          </div>
        </div>
      </div>

      <div class="profile-posts">
        <h3>Your Posts</h3>
        <Show
          when={ownEvents().length > 0}
          fallback={<p class="empty-hint">You haven't posted anything yet.</p>}
        >
          <For each={ownEvents()}>
            {(event) => (
              <div class="profile-post">
                <p class="profile-post-content">{event.content}</p>
                <span class="profile-post-time">{event.time_ago}</span>
              </div>
            )}
          </For>
        </Show>
      </div>
    </div>
  );
}
