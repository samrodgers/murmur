/** Matches EventDto from the Tauri backend. */
export interface MurmurEvent {
  id: string;
  pubkey: string;
  pubkey_short: string;
  created_at: number;
  kind: number;
  kind_label: string;
  content: string;
  parent_id: string | null;
  thread_root_id: string | null;
  is_own: boolean;
  time_ago: string;
}

export interface Identity {
  pubkey: string;
  pubkey_short: string;
}

export interface Stats {
  total_events: number;
  cached_events: number;
  own_events: number;
  peer_count: number;
}

export interface Thread {
  root: MurmurEvent | null;
  replies: MurmurEvent[];
}

export type View = "timeline" | "thread" | "profile";
