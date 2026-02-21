import { createSignal, Show } from "solid-js";
import { publishPost, publishReply } from "../commands";
import type { MurmurEvent } from "../types";

const MAX_LENGTH = 500;

interface ComposeProps {
  replyTo: MurmurEvent | null;
  onClose: () => void;
  onSuccess: () => void;
}

export default function Compose(props: ComposeProps) {
  const [content, setContent] = createSignal("");
  const [sending, setSending] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const remaining = () => MAX_LENGTH - content().length;
  const canSend = () => content().trim().length > 0 && remaining() >= 0 && !sending();

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!canSend()) return;

    setSending(true);
    setError(null);

    try {
      if (props.replyTo) {
        await publishReply(content().trim(), props.replyTo.id);
      } else {
        await publishPost(content().trim());
      }
      props.onSuccess();
    } catch (err) {
      setError(String(err));
    } finally {
      setSending(false);
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    // Cmd/Ctrl+Enter to submit
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      handleSubmit(e);
    }
    // Escape to close
    if (e.key === "Escape") {
      props.onClose();
    }
  }

  return (
    <div class="compose-overlay" onClick={props.onClose}>
      <div class="compose-modal" onClick={(e) => e.stopPropagation()}>
        <div class="compose-header">
          <h3>{props.replyTo ? "Reply" : "New Post"}</h3>
          <button class="close-btn" onClick={props.onClose}>
            &times;
          </button>
        </div>

        <Show when={props.replyTo}>
          {(parent) => (
            <div class="reply-context">
              <span class="reply-label">Replying to</span>
              <span class="reply-author">{parent().pubkey_short}</span>
              <p class="reply-preview">{parent().content.slice(0, 100)}...</p>
            </div>
          )}
        </Show>

        <form onSubmit={handleSubmit}>
          <textarea
            class="compose-input"
            placeholder={
              props.replyTo
                ? "Write your reply..."
                : "What's on your mind?"
            }
            value={content()}
            onInput={(e) => setContent(e.currentTarget.value)}
            onKeyDown={handleKeyDown}
            maxLength={MAX_LENGTH}
            autofocus
            rows={5}
          />

          <div class="compose-footer">
            <span class={`char-count ${remaining() < 50 ? "warn" : ""} ${remaining() < 0 ? "over" : ""}`}>
              {remaining()}
            </span>

            <Show when={error()}>
              <span class="compose-error">{error()}</span>
            </Show>

            <div class="compose-actions">
              <span class="compose-hint">Ctrl+Enter to send</span>
              <button
                type="submit"
                class="send-btn"
                disabled={!canSend()}
              >
                {sending() ? "Sending..." : props.replyTo ? "Reply" : "Post"}
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}
