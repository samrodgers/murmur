import { listen } from "@tauri-apps/api/event";
import type { Post } from "./types";

export interface PeerChangedPayload {
  connected_count: number;
}

export function onNewPost(callback: (post: Post) => void) {
  return listen<Post>("new-post", (event) => {
    callback(event.payload);
  });
}

export function onPeerChanged(callback: (payload: PeerChangedPayload) => void) {
  return listen<PeerChangedPayload>("peer-changed", (event) => {
    callback(event.payload);
  });
}

export function onTemperatureChanged(
  callback: (payload: { post_id: string; temperature: number }) => void,
) {
  return listen<{ post_id: string; temperature: number }>(
    "temperature-changed",
    (event) => {
      callback(event.payload);
    },
  );
}

export function onNetworkStatusChanged(
  callback: (payload: { online: boolean }) => void,
) {
  return listen<{ online: boolean }>("network-status-changed", (event) => {
    callback(event.payload);
  });
}
