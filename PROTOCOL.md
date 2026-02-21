# Protocol Primitives — Murmur

*A serverless, decentralised social discussion protocol where the network breathes with attention.*

---

## 1. Core Concepts

### The Party Model

- Your phone is your identity, your archive, and a node in the network
- Posts are spoken into the room — they spread as far as interest carries them
- Nothing is deleted; everything cools naturally
- Trust is earned through interaction, not granted at signup

### Temperature

A single unified metric that governs both content distribution and identity reputation. Temperature is a product of **recency** and **current engagement**. Hot = widely replicated. Cold = lives only on the author's device. Old things can reheat if people engage with them again.

---

## 2. Identity

### Account Creation

- Device generates an **Ed25519 keypair** locally
- Public key = account ID (canonical, permanent)
- Private key = never leaves the device (stored in secure enclave / keychain)
- A **proof-of-work challenge** is computed at registration (e.g. find a partial hash collision, ~30 seconds on a phone) — this is the sybil resistance layer
- The PoW result + public key are published to the DHT as a **registration event**
- User chooses a **display name** (not globally unique, not verified — like a party name tag)

### Key Backup

- Encrypted backup to iCloud/Google Drive (transparent to user)
- Recovery via backup restores full identity
- If keys are lost with no backup, the account is gone (like losing your house keys)

### Identity Temperature

- New accounts start cold (effectively invisible to the broader network)
- Engagement from warmer accounts raises your temperature
- Your identity temperature affects the initial distribution radius of your posts
- Persistent bad behaviour (mass-blocking by others) cools you toward zero

---

## 3. Events

Everything on the network is a **signed event**. All events share a common envelope:

```
Event {
    id:           SHA-256 hash of the serialised content below
    pubkey:       author's public key
    created_at:   unix timestamp
    kind:         integer event type
    content:      event-specific payload (string)
    sig:          Ed25519 signature of the id
    pow:          proof-of-work nonce (registration events only)
}
```

### Event Kinds

| Kind | Name          | Description                                      |
|------|---------------|--------------------------------------------------|
| 0    | Registration  | Account creation (includes PoW proof)            |
| 1    | Post          | A top-level post (max 500 chars)                 |
| 2    | Reply         | A response to a post (references parent event)   |
| 3    | Repost        | Amplification of another post                    |
| 4    | Reaction      | Lightweight engagement (like, etc.)              |
| 5    | Profile       | Profile metadata update (name, bio, avatar hash) |
| 6    | Follow        | Declares interest in another pubkey              |
| 7    | Unfollow      | Revokes follow                                   |
| 8    | Block         | Declares you don't want content from a pubkey    |
| 9    | Correction    | References an earlier post with amended content  |

---

## 4. Post Lifecycle

### Publishing

1. Author creates and signs a Kind 1 (Post) event
2. Event is stored locally on the author's device (permanent)
3. Event is pushed to **connected peers** in the author's DHT neighbourhood
4. Initial replication target: **10–15 nodes** (baseline warmth)

### Heating

- Each engagement event (reply, reaction, repost) increments the post's temperature
- As temperature rises, the post is **proactively replicated** to more nodes
- Hot post: replicated across 50–100+ nodes, highly discoverable
- The replication is pull-based: nodes in the DHT observe rising temperature metadata and request the content

### Cooling

- Temperature decays over time (half-life ~14 hours)
- As temperature drops, nodes **stop actively serving** the post (but don't delete cached copies immediately)
- Eventually only the author's device and a small number of stale caches hold the post
- If engagement resurfaces (someone replies, reposts), temperature rises again and redistribution begins

### Temperature Formula

```
T(post) = Σ(engagement_weight × recency_factor)

where:
  engagement_weight:
    reaction  = 1
    reply     = 3
    repost    = 5

  recency_factor = e^(-λt)
    t = time since engagement event (hours)
    λ = 0.05 (half-life ~14 hours)
```

Replication target = `min(max(T × k, FLOOR), CEILING)`

- FLOOR = 3 (minimum copies for any live post)
- CEILING = 200 (cap to prevent runaway replication)
- k = scaling factor (tunable)

---

## 5. Threads

### Structure

- A Reply (Kind 2) includes a `parent_id` field referencing the parent event's ID
- The **OP's device** stores the full thread: the original post plus all replies
- Each **replier's device** also stores their own reply with the `parent_id` pointer
- Two paths to any reply: via the OP's thread, or via the replier's profile

### Thread Temperature

- A thread's temperature is the **aggregate** of its component events
- Hot threads are replicated as a unit (the OP's device serves the bundle)
- Individual replies within a cold thread can independently reheat the whole thread

---

## 6. Networking

### DHT (Distributed Hash Table)

- Kademlia-based DHT for peer discovery and content routing
- Each node maintains a routing table of known peers
- Content is addressable by event ID (SHA-256 hash)
- Nodes also maintain a **temperature index** — a lightweight metadata layer tracking the temperature of events they're aware of

### Peer Discovery

- Bootstrap nodes for initial network entry (hardcoded list shipped with app for MVP)
- After bootstrap, peer discovery is fully decentralised via DHT
- Phase 1 uses mDNS for local network discovery

### Offline Handling

- Your device going offline doesn't affect others' access to hot content (it's replicated elsewhere)
- Your cold/old posts are unavailable when you're offline — this is by design
- When you come back online, your node re-announces itself and begins serving again

---

## 7. Identity Temperature & Trust

### Web of Trust

- No explicit verification step
- Trust emerges from interaction patterns
- Your identity temperature is a function of:
  - How many warm accounts engage with you
  - How long you've been active
  - Your ratio of engagement-received to content-posted (spam signal)

### Sybil Resistance (Layered)

1. **PoW at registration** — creating an account costs ~30 seconds of computation
2. **Cold start** — new accounts are invisible until warmed by genuine engagement
3. **Engagement-weighted trust** — a reaction from a high-temperature account warms you more than one from a cold account

### Abuse Handling

- No global moderation — each client can implement its own filtering
- Block events (Kind 8) are signed and public — clients can aggregate shared blocklists
- A widely-blocked account's temperature drops (blocks count as negative engagement)

---

## 8. Feed Construction

### Following Feed

- Client maintains a local list of followed pubkeys
- Queries the DHT for recent events from followed pubkeys
- Assembles timeline locally, sorted by recency (not algorithmic ranking)

### Discovery Feed

- Client queries connected peers for **high-temperature events** across the network
- No algorithm decides what's "interesting" — temperature *is* the signal

### "New Voices" Feed

- Surfaces accounts below a temperature threshold that have recent activity
- Gives cold-start users a chance to be discovered

---

## 9. Storage Budget

### On-Device Storage

- **Your own content**: stored permanently
- **Others' content you're caching**: governed by a storage budget (default 500MB)
- Cache eviction follows temperature: coldest content is evicted first
- Users can opt out of caching entirely (freeloading) but this weakens the network

---

## 10. Design Decisions

1. **Character limit**: 500 characters per post. Thread for longer thoughts.
2. **Media**: Text-only for v1. URLs in post content parsed by client for link previews.
3. **Direct messages**: Out of scope for the base protocol.
4. **Temperature decay**: λ = 0.05 (half-life ~14 hours). Tune through real-world testing.
5. **Bootstrap nodes**: Hardcoded list for MVP. DNS-based discovery and manual peer entry as future fallback layers.
6. **Cross-platform sync**: Deferred. Single-device only for MVP.
7. **Post deletion**: Not supported. Once posted, it cools naturally. Corrections (Kind 9) allow amending.
8. **Engagement gaming**: Cold-account weighting provides baseline mitigation. Revisit when real usage patterns emerge.
