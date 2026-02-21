import { createSignal, onMount, onCleanup, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import { getIdentity, getTimeline, getStats } from "./commands";
import type { MurmurEvent, Identity, Stats, View } from "./types";
import Timeline from "./components/Timeline";
import ThreadView from "./components/ThreadView";
import ProfileView from "./components/ProfileView";
import Compose from "./components/Compose";
import StatusBar from "./components/StatusBar";

export default function App() {
  const [identity, setIdentity] = createSignal<Identity | null>(null);
  const [events, setEvents] = createSignal<MurmurEvent[]>([]);
  const [stats, setStats] = createSignal<Stats>({
    total_events: 0,
    cached_events: 0,
    own_events: 0,
    peer_count: 0,
  });
  const [view, setView] = createSignal<View>("timeline");
  const [threadId, setThreadId] = createSignal<string | null>(null);
  const [showCompose, setShowCompose] = createSignal(false);
  const [replyTo, setReplyTo] = createSignal<MurmurEvent | null>(null);
  const [statusMsg, setStatusMsg] = createSignal<string | null>(null);

  async function refresh() {
    try {
      const [tl, st] = await Promise.all([getTimeline(), getStats()]);
      setEvents(tl);
      setStats(st);
    } catch (e) {
      console.error("Failed to refresh:", e);
    }
  }

  onMount(async () => {
    // Small delay to let Tauri state initialize
    await new Promise((r) => setTimeout(r, 500));

    try {
      const id = await getIdentity();
      setIdentity(id);
    } catch (e) {
      console.error("Failed to get identity:", e);
    }

    await refresh();

    // Listen for real-time events from the Tauri backend
    const unlistenEvent = await listen<MurmurEvent>("new-event", (e) => {
      setEvents((prev) => [e.payload, ...prev]);
      setStatusMsg("New event received");
      setTimeout(() => setStatusMsg(null), 3000);
    });

    const unlistenPeerCount = await listen<number>("peer-count", (e) => {
      setStats((prev) => ({ ...prev, peer_count: e.payload }));
    });

    const unlistenPeerDiscovered = await listen<string>(
      "peer-discovered",
      (e) => {
        setStatusMsg(`Peer discovered: ${e.payload.slice(0, 16)}...`);
        setTimeout(() => setStatusMsg(null), 4000);
      }
    );

    const unlistenPeerLost = await listen<string>("peer-lost", (e) => {
      setStatusMsg(`Peer disconnected: ${e.payload.slice(0, 16)}...`);
      setTimeout(() => setStatusMsg(null), 4000);
    });

    onCleanup(() => {
      unlistenEvent();
      unlistenPeerCount();
      unlistenPeerDiscovered();
      unlistenPeerLost();
    });

    // Periodic refresh
    const interval = setInterval(refresh, 10000);
    onCleanup(() => clearInterval(interval));
  });

  function openThread(eventId: string) {
    setThreadId(eventId);
    setView("thread");
  }

  function openReply(event: MurmurEvent) {
    setReplyTo(event);
    setShowCompose(true);
  }

  function handleBack() {
    setView("timeline");
    setThreadId(null);
  }

  function handlePostSuccess() {
    setShowCompose(false);
    setReplyTo(null);
    refresh();
    setStatusMsg("Published!");
    setTimeout(() => setStatusMsg(null), 3000);
  }

  return (
    <div class="app">
      {/* Header */}
      <header class="header">
        <div class="header-left">
          <h1 class="logo" onClick={() => handleBack()}>
            murmur
          </h1>
        </div>
        <nav class="header-nav">
          <button
            class={view() === "timeline" ? "nav-btn active" : "nav-btn"}
            onClick={() => handleBack()}
          >
            Timeline
          </button>
          <button
            class={view() === "profile" ? "nav-btn active" : "nav-btn"}
            onClick={() => setView("profile")}
          >
            Profile
          </button>
        </nav>
        <div class="header-right">
          <button class="compose-btn" onClick={() => setShowCompose(true)}>
            New Post
          </button>
        </div>
      </header>

      {/* Main Content */}
      <main class="main">
        <Show when={view() === "timeline"}>
          <Timeline
            events={events()}
            onOpenThread={openThread}
            onReply={openReply}
          />
        </Show>
        <Show when={view() === "thread" && threadId()}>
          <ThreadView
            eventId={threadId()!}
            onBack={handleBack}
            onReply={openReply}
            ownPubkey={identity()?.pubkey ?? ""}
          />
        </Show>
        <Show when={view() === "profile"}>
          <ProfileView
            identity={identity()}
            stats={stats()}
            events={events()}
          />
        </Show>
      </main>

      {/* Compose Modal */}
      <Show when={showCompose()}>
        <Compose
          replyTo={replyTo()}
          onClose={() => {
            setShowCompose(false);
            setReplyTo(null);
          }}
          onSuccess={handlePostSuccess}
        />
      </Show>

      {/* Status Bar */}
      <StatusBar
        peerCount={stats().peer_count}
        identity={identity()}
        statusMsg={statusMsg()}
      />
    </div>
  );
}
