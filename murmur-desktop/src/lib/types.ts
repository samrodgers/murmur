export interface Post {
  id: string;
  author: Profile;
  content: string;
  created_at: number;
  temperature: number;
  reply_count: number;
  repost_count: number;
  reaction_count: number;
  is_reply: boolean;
  parent_id?: string;
  has_reacted: boolean;
  has_reposted: boolean;
}

export interface Profile {
  pubkey: string;
  name: string;
  bio: string;
  temperature: number;
  post_count: number;
  joined_at: number;
  is_following: boolean;
  is_blocked: boolean;
  is_online: boolean;
  avatar_colour: string;
}

export interface Thread {
  root: Post;
  replies: Post[];
}

export interface NetworkStatus {
  online: boolean;
  peer_count: number;
  peers: PeerInfo[];
  error?: string;
}

export interface StorageStats {
  own_posts: number;
  cached_posts: number;
  storage_used_mb: number;
  cache_limit_mb: number;
}

export interface PeerInfo {
  peer_id: string;
  display_name?: string;
  connected_since: number;
}

export type FeedTab = "following" | "discover" | "new-voices";

export type Route =
  | { page: "onboarding" }
  | { page: "timeline" }
  | { page: "thread"; postId: string }
  | { page: "profile"; pubkey: string }
  | { page: "settings" };
