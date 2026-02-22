/** Format a unix timestamp as a relative time string. */
export function relativeTime(timestamp: number): string {
  const now = Math.floor(Date.now() / 1000);
  const diff = now - timestamp;

  if (diff < 0) return "just now";
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
  return new Date(timestamp * 1000).toLocaleDateString();
}

/** Truncate a pubkey for display: "abcdef12…" */
export function shortPubkey(pubkey: string): string {
  if (pubkey.length <= 12) return pubkey;
  return pubkey.slice(0, 8) + "…";
}

/** Get the first letter (uppercase) for a letter avatar. */
export function avatarLetter(name: string): string {
  return (name[0] || "?").toUpperCase();
}
