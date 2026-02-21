import { Show } from "solid-js";
import type { Identity } from "../types";

interface StatusBarProps {
  peerCount: number;
  identity: Identity | null;
  statusMsg: string | null;
}

export default function StatusBar(props: StatusBarProps) {
  return (
    <footer class="status-bar">
      <div class="status-left">
        <span class={`peer-indicator ${props.peerCount > 0 ? "connected" : ""}`}>
          {props.peerCount > 0 ? "\u25CF" : "\u25CB"}
        </span>
        <span class="peer-count">
          {props.peerCount} peer{props.peerCount !== 1 ? "s" : ""}
        </span>
      </div>

      <Show when={props.statusMsg}>
        <div class="status-center">
          <span class="status-msg">{props.statusMsg}</span>
        </div>
      </Show>

      <div class="status-right">
        <span class="identity-badge">{props.identity?.pubkey_short ?? "..."}</span>
      </div>
    </footer>
  );
}
