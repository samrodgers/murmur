import { invoke } from "@tauri-apps/api/core";
import type { MurmurEvent, Identity, Stats, Thread } from "./types";

export async function getIdentity(): Promise<Identity> {
  return invoke("get_identity");
}

export async function getTimeline(): Promise<MurmurEvent[]> {
  return invoke("get_timeline");
}

export async function getPosts(): Promise<MurmurEvent[]> {
  return invoke("get_posts");
}

export async function getThread(eventId: string): Promise<Thread> {
  return invoke("get_thread", { eventId });
}

export async function publishPost(content: string): Promise<MurmurEvent> {
  return invoke("publish_post", { content });
}

export async function publishReply(
  content: string,
  parentId: string
): Promise<MurmurEvent> {
  return invoke("publish_reply", { content, parentId });
}

export async function getStats(): Promise<Stats> {
  return invoke("get_stats");
}

export async function getPeerCount(): Promise<number> {
  return invoke("get_peer_count");
}
