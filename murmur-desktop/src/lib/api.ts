import { invoke } from "@tauri-apps/api/core";
import type {
  Post,
  Profile,
  Thread,
  NetworkStatus,
  StorageStats,
} from "./types";

// ─── Identity ──────────────────────────────────────────────────────────

export async function hasIdentity(): Promise<boolean> {
  return invoke("has_identity");
}

export async function createIdentity(
  name: string,
  bio: string,
): Promise<Profile> {
  return invoke("create_identity", { name, bio });
}

export async function getOwnProfile(): Promise<Profile> {
  return invoke("get_own_profile");
}

export async function updateProfile(
  name: string,
  bio: string,
): Promise<Profile> {
  return invoke("update_profile", { name, bio });
}

export async function exportIdentity(
  path: string,
  passphrase: string,
): Promise<void> {
  return invoke("export_identity", { path, passphrase });
}

export async function importIdentity(
  path: string,
  passphrase: string,
): Promise<Profile> {
  return invoke("import_identity", { path, passphrase });
}

// ─── Posts ──────────────────────────────────────────────────────────────

export async function createPost(content: string): Promise<Post> {
  return invoke("create_post", { content });
}

export async function createReply(
  content: string,
  parentId: string,
): Promise<Post> {
  return invoke("create_reply", { content, parentId });
}

export async function reactToPost(postId: string): Promise<void> {
  return invoke("react_to_post", { postId });
}

export async function repostPost(postId: string): Promise<void> {
  return invoke("repost", { postId });
}

// ─── Feeds ─────────────────────────────────────────────────────────────

export async function getFollowingFeed(limit: number = 50): Promise<Post[]> {
  return invoke("get_following_feed", { limit });
}

export async function getDiscoverFeed(limit: number = 50): Promise<Post[]> {
  return invoke("get_discover_feed", { limit });
}

export async function getNewVoicesFeed(limit: number = 50): Promise<Post[]> {
  return invoke("get_new_voices_feed", { limit });
}

export async function getThread(postId: string): Promise<Thread> {
  return invoke("get_thread", { postId });
}

// ─── Social ────────────────────────────────────────────────────────────

export async function followUser(pubkey: string): Promise<void> {
  return invoke("follow_user", { pubkey });
}

export async function unfollowUser(pubkey: string): Promise<void> {
  return invoke("unfollow_user", { pubkey });
}

export async function blockUser(pubkey: string): Promise<void> {
  return invoke("block_user", { pubkey });
}

export async function getProfile(pubkey: string): Promise<Profile> {
  return invoke("get_profile", { pubkey });
}

export async function getFollowingList(): Promise<Profile[]> {
  return invoke("get_following_list");
}

// ─── Network ───────────────────────────────────────────────────────────

export async function getNetworkStatus(): Promise<NetworkStatus> {
  return invoke("get_network_status");
}

export async function startNetwork(): Promise<NetworkStatus> {
  return invoke("start_network");
}

export async function getStorageStats(): Promise<StorageStats> {
  return invoke("get_storage_stats");
}

export async function setCacheLimit(megabytes: number): Promise<void> {
  return invoke("set_cache_limit", { megabytes });
}
