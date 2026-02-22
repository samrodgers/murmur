import type { Post, Profile, Route, FeedTab } from "./types";

// Svelte 5 uses $state runes instead of stores.
// We use a simple reactive state object that components import.

class AppStore {
  route: Route = $state({ page: "onboarding" });
  profile: Profile | null = $state(null);
  timeline: Post[] = $state([]);
  currentTab: FeedTab = $state("following");
  peerCount: number = $state(0);
  isOnline: boolean = $state(false);

  navigate(route: Route) {
    this.route = route;
  }

  setProfile(profile: Profile) {
    this.profile = profile;
  }

  addPost(post: Post) {
    // Add to front, avoid duplicates
    if (!this.timeline.find((p) => p.id === post.id)) {
      this.timeline = [post, ...this.timeline];
    }
  }

  setTimeline(posts: Post[]) {
    this.timeline = posts;
  }

  setOnline(online: boolean) {
    this.isOnline = online;
  }

  setPeerCount(count: number) {
    this.peerCount = count;
  }

  removePost(postId: string) {
    this.timeline = this.timeline.filter((p) => p.id !== postId);
  }

  updatePostReactions(postId: string, count: number) {
    const post = this.timeline.find((p) => p.id === postId);
    if (post) {
      post.reaction_count = count;
      post.has_reacted = true;
    }
  }
}

export const store = new AppStore();
