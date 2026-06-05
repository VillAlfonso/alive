# ALIVE — NPC Social & Psychology Roadmap

A living document for the parts of *Alive* not yet built. The guiding rule for all of it:

> **Simulation owns reality. The model is the mouth.**
> Cheap code decides *what is true*, *who acts*, and *whether* they act.
> The language model is only ever asked to provide *words*, and only for **one** NPC at a time.

If a feature would require the model to run for many NPCs at once, it's designed wrong. Re-shape it so simulation does the deciding and the model only speaks.

---

## The core principle: layering

Every living thing shares one cheap **survival base** (needs → decide → act). Specialization is *layered on top* only for creatures that need it.

- **Animal** = base only (hunger, flee, wander).
- **Smart creature / human NPC** = base + memory + relationships + speech + culture.

Same engine underneath; the expensive layers (memory, the model) run only for the handful of NPCs that earn them. This is what keeps a crowded world cheap.

---

## Status (what already works)

- NPCs with draining needs (hunger, thirst), choosing the loudest need, acting on it.
- Idle wandering when content; seeking a content friend to stand near (first social pull).
- Ambient speech bubbles driven by NPC state (cheap, templated lines + emotion color).
- Directed one-on-one dialogue (press F near an NPC): real local-LLM replies via Ollama, on a background thread so the frame never stalls.
- Player can type a reply (T) to the NPC, or broadcast/shout (T in open space) — shout currently shows the player's words as a floating bubble.
- Two named NPCs with distinct hand-written personalities + memories (Bram the smith, Senna the herbalist).

---

## NEXT BRICK: broadcast reactions

**Goal:** when the player shouts, nearby NPCs *hear* it and the single most-interested one can respond (or approach).

**Design (decided):**
- There is **no central observer entity.** Each NPC is its own reader. A broadcast is an **event**; every NPC in range judges it against *its own* state.
- **Two stages, to protect performance:**
  1. **Cheap filter (simulation, runs for everyone):** is the NPC in range? not busy / not panicking? mood + sociability high enough? relationship to the speaker positive enough? → produces an "interest score."
  2. **Expensive voice (model, runs for at most ONE):** the highest-interest NPC above a threshold responds with a real generated line. Everyone else either ignores it or emits a cheap canned reaction (a grunt, a shrug emote, a templated bubble).
- Optionally, a sufficiently-interested NPC **approaches** the player instead of (or before) speaking.

**Smallest provable slice to build first:** broadcast becomes an event; nearby NPCs run the cheap interest filter; the single most-interested one calls the model and replies in a bubble. No deep psychology yet.

**Why it's structured this way:** five NPCs each calling the model to "decide whether to care" would melt the machine. Simulation decides *who* and *whether* for free; the model speaks for one.

---

## Deeper psychology (later, layered on)

These make the interest filter and dialogue richer over time. Each is a number or small system the simulation owns; the model just *reads* the resulting state as "stage directions."

- **Mood** — drifts toward how secure/fed/safe the NPC feels; gates willingness to socialize, talk, join in.
- **Personality traits** (sociability, bravery, curiosity, warmth, openness) — set at spawn from circumstance; bias every decision. High-openness NPCs more likely to start/adopt new things (→ culture).
- **Relationships** — per-other-entity bond (affection, trust), nudged by shared time and events. Drives who they help, trust, approach, believe.
- **Memory** — append-only life-log on disk; only the few *relevant* entries streamed into a conversation's context (the `recall()` problem — the real engineering challenge of the whole AI layer).
- **Willingness to talk** — survival panic = hard veto; otherwise a weighted sum of sociability + mood + bond + self-interest vs a threshold. (This also gates the broadcast filter and the "do they accept a one-on-one" handshake.)
- **Emotion display** — shown via bubble color, a small emote icon, and body language (sprite flash/shake/bounce), not heavy portraits.

---

## Social activities & "life" (later)

When survival pressure is low and the area is safe, NPCs pursue *life*, not just survival. The action scorer simply gains more options:

- hang out with liked people, play, train, tell stories, celebrate at gatherings/parties (a world event with a location + mood boost; nearby NPCs swap to festive emotes/lines).
- **structured games** (e.g. Mafia) as *coded activity systems* — roles, phases, suspicion/trust math — that the model dresses in language, never invents wholesale.

---

## Culture (much later, emergent)

Not authored — it *accumulates*. Open NPCs invent variations (a greeting, a song, a rule tweak); liked NPCs get copied (word-of-mouth / teaching = knowledge copying between minds); copied enough, it becomes "how this village does things." Culture is just **behaviors that spread and stuck** — the same memory-copying machinery that turns facts into myths, pointed at customs.

---

## The "talk to anyone" handshake (later)

- Player requests a one-on-one (F near NPC) → NPC **decides** whether to accept, based on willingness (mood + bond + busy?). Can brush you off, flavored by *why*.
- An NPC who wants the player pops a request indicator → player accepts with F.
- Same speech, different *range*: a shout is broadcast (everyone nearby), a one-on-one is a private channel. One bubble system; the conversation decides who hears.

---

## Lore & world generation (Phase 2)

- A settlement = a handful of generated numbers (wealth, age, safety, size, **water access**). People are generated *downstream* of those numbers — circumstance shapes personality, not the reverse.
- **Water access is a settlement trait, not a global utility.** Place settlements where water is (river > well > cistern > engineered aqueduct/qanat). Scarcity becomes character (precarious, wary villages). Bringing water to a dry settlement is a concrete "unite the frontier" power move.
- Lore (names, history, legend) generated offline at gen time and **frozen to data**; the model writes it once, then it's just facts NPCs can know, share, and mutate into myth over distance.

---

## Setting (locked)

**Low-magic frontier wilderness, dotted with the ruins of a fallen age.**
- Separated settlements (procedural gen + the "you are the bridge between them" fantasy both work).
- No central power yet → you can found/unite a kingdom (matches conquest + building hooks).
- Lawless → crime & conquest work. Raw land → farming & mining work. Buried history → mystery & dungeon-delving.
- Frontier *energy* (hopeful, upward) on the *bones* of something older (haunted, mysterious) = best of both without committing to pure optimism or pure melancholy.

---

## Shipping the local model (Phase 3 — not now)

The single unsolved-for-consumers problem. Options, worst → best for a shipped game:
1. Require players to install Ollama — **dead on arrival**, don't.
2. **Bundle** Ollama + model with the game, launch it silently — works, but multi-GB download + managing a child process.
3. **Embed** the model in the Rust binary (`candle`, `llama-cpp` bindings, `mistral.rs`) — no install, model is just a file the game loads. The right long-term answer; harder to set up.
4. **Cloud** — sidesteps install entirely (what most polished commercial AI-NPC games do), but costs money per call + needs internet.

Build against Ollama-over-HTTP now (already done), hidden behind one swappable function, so the backend can change later without touching game logic. Decide bundling vs embedding vs cloud only once the feature is proven fun.

---

## Performance reminders (keep these true forever)

- Most NPCs run **only** the cheap simulation base. The model touches **one** NPC at a time.
- Any slow work (model calls, saving, world-gen, big pathfinding) runs on a **background thread + channel**, never on the frame loop.
- Ambient chatter, broadcast "hearing," interest filtering = simulation, free. Generated *words* = rare, one speaker.
- Small fast model (3B) for in-world chatter; reserve any bigger/slower model for rare deep moments.
