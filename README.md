# 🧹 Chore Scheduler

A fair chore rotation scheduler for shared households, written in Rust.

## Problem

Roommates need to split chores fairly, but naive rotation breaks down when people have different availability (work schedules, travel, etc.). This scheduler handles that by combining multiple fairness heuristics into a single assignment algorithm.

## Fairness Rules

The scheduler enforces four rules when assigning chores:

1. **No consecutive repeats** — Nobody gets stuck doing the same chore back-to-back.
2. **Workload balancing** — People who've done fewer chores overall are prioritized.
3. **Cooldown period** — Recently assigned people get a 2-round cooldown to prevent burnout from being assigned every single day.
4. **Availability constraints** — Respects each person's declared unavailable days (e.g., work shifts).

## Usage

```bash
# Run the 9-week simulation demo
cargo run

# Run tests
cargo test

# Run with output shown
cargo test -- --nocapture
```

## Architecture

```
src/
├── lib.rs    # Core Scheduler struct and assignment logic
└── main.rs   # Demo: 9-week simulation with 3 roommates

tests/
└── integration_tests.rs   # 13 integration tests
```

### Key Types

- **`Person`** — Represents a roommate (identified by name).
- **`Scheduler`** — The main struct holding assignment history, cooldowns, and unavailability data.
- **`Assignment`** — A record of who was assigned which chore.

## Design Decisions

### Why cooldown is the primary sort key

The scheduler sorts candidates by `(cooldown, completed_count)`. This means recency is weighted more heavily than total workload. The reasoning:

- **Prevents burnout**: Even if someone has done fewer chores overall, assigning them three days in a row feels unfair *in the moment*. The cooldown ensures everyone gets breathing room.
- **Natural round-robin emergence**: With a cooldown of 2 and 3 people, the system naturally cycles through all participants before repeating.
- **Workload still balances as a tiebreaker**: Among people with equal cooldown (i.e., equally rested), the one with fewer total chores gets priority.

### Bounded history window

Assignment history is capped at 30 entries to keep memory usage constant. This is sufficient to detect consecutive-repeat violations without unbounded growth.

## Example Output

```
=== Chore Scheduler: 9-Week Simulation ===
Rules:
  1. No one repeats the same chore consecutively
  2. Fewer total chores → higher priority
  3. Recently assigned people get a 2-round cooldown
  4. Cara is unavailable Mon/Tue/Wed (late shifts)

--- Week 1 ---
  monday (dishes):  → Alice
  tuesday (trash):  → Bob
  wednesday (sweeping):  → Alice
  thursday (dishes):  → Cara
  ...
```

## Future Work

- [ ] Add a `clear_unavailability` method for dynamic schedule changes
- [ ] Support chore preferences / weights (some chores take longer)
- [ ] Web UI or CLI interactive mode
- [ ] Persistent state (save/load schedule to file)

## License

MIT
