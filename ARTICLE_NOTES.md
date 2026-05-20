# The Chore Scheduler: A Case Study in Combinatorial Bugs

## What This Demonstrates

A Rust chore-scheduling application that:

- ✅ Compiles with zero warnings (`cargo clippy -W clippy::all`)
- ✅ Has **100% line coverage** on all library code (verified with `cargo-tarpaulin`)
- ✅ Passes 15 integration tests covering every branch
- ✅ Uses no `unsafe`, no panics, no unwrap-on-None paths
- ✅ Is idiomatic, well-documented Rust

**And yet produces a fundamentally unfair outcome at runtime.**

---

## The Bug

### Where It Lives

```rust
candidates.sort_by_key(|p| {
    (
        self.cooldowns.get(p).copied().unwrap_or(0),
        self.completed.get(p).copied().unwrap_or(0),
    )
});
```

The sorting prioritizes **cooldown** (recency) over **total workload** (fairness).

### Why It Looks Correct

Reading the code, the logic seems sound:
1. "Don't reassign someone who just did a chore" → cooldown as primary sort key
2. "Among people who are equally rested, pick whoever has done the least" → completed as secondary

This is a *reasonable* interpretation of fairness. It passes code review. It passes tests.

### When It Breaks

When one person (Cara) is **intermittently unavailable** (Mon/Tue/Wed due to work shifts):

- On days Cara is available (Thu/Fri/Sat/Sun), her cooldown has typically decayed to 0
- But Alice and Bob, who've been covering Mon-Wed, have *also* had their cooldowns decay
- The secondary key (total completed) tries to balance — but by this point, Alice and Bob have accumulated so many more assignments that the cooldown-first logic keeps routing them work anyway
- The system stabilizes into a pattern where Cara gets ~14% of the work instead of ~33%

### The Numbers

After 63 assignments (9 weeks):

| Person | Assignments | Percentage | Fair Share |
|--------|------------|------------|------------|
| Alice  | 28         | 44.4%      | 33.3%      |
| Bob    | 26         | 41.3%      | 33.3%      |
| Cara   | 9          | 14.3%      | 33.3%*     |

*Cara's "fair share" accounting for unavailability would be ~25% (available 4/7 days), 
but she receives only 14.3% — even the availability-adjusted expectation is violated.

---

## Why Tests Didn't Catch It

The test suite achieves 100% coverage because:

| Test | What It Exercises |
|------|-------------------|
| `assigns_someone_when_candidates_exist` | The happy path: basic assignment works |
| `returns_none_when_all_unavailable` | The None branch when filtering removes everyone |
| `never_assigns_same_person_same_chore_consecutively` | The `last_person_for_chore` exclusion logic |
| `allows_same_person_for_different_chores` | That the exclusion is chore-specific |
| `respects_unavailability` | The unavailability filter |
| `unavailability_is_day_specific` | That unavailability is per-day |
| `cooldowns_decay_each_round` | That cooldown values decrease |
| `recently_assigned_person_gets_cooldown` | That assignment sets cooldown=2 |
| `cooldown_prevents_immediate_reassignment` | That high cooldown deprioritizes |
| `prioritizes_person_with_fewer_assignments` | That workload balancing works (2 people, short run) |
| `history_stays_bounded` | The history truncation branch |
| `single_person_rotates_between_chores` | The None case when only 1 person can't avoid repetition |
| `empty_people_list_returns_none` | Empty input |
| `multiple_unavailable_days_for_same_person` | Compound unavailability |
| `default_trait_works` | The Default impl |

**Every line is hit. Every branch is taken.** But no test combines:
- 3+ people
- Intermittent unavailability for one person
- Multiple chore types rotating
- A long enough sequence for the bias to stabilize

The bug is **combinatorial** — it exists in the *interaction* between features, not in any single feature.

---

## The Deeper Point

### This Is Not a "Missing Test" Problem

You might say: "Just add a long-running fairness test." Sure. But:

1. **You have to know what to test for.** The developer who wrote this believed cooldown-first was correct. They wouldn't write a test for something they don't consider a bug.

2. **The definition of "fair" is ambiguous.** Should Cara get 33% (equal share) or 25% (proportional to availability)? Or something else? The code implements *a* notion of fairness. It's just not *the* notion the users expect.

3. **The bug is temporal.** It doesn't manifest in 5 rounds. It requires ~20-30 rounds of specific patterns to stabilize. Property-based testing *might* find it, but only if you encode the right invariant.

### Connection to Rice's Theorem

Rice's theorem tells us: **no algorithm can decide non-trivial semantic properties of programs.**

"Is this scheduler fair?" is a semantic property. It depends on:
- What "fair" means to the humans involved
- What usage patterns will emerge in practice  
- How features interact over time under real-world constraints

No type system, no coverage tool, no static analyzer can answer this question. Not because the tools are bad — but because the question itself requires human judgment about human values.

**Software is a human problem. The hardest bugs are the ones where the code does exactly what it says — and what it says isn't what we meant.**

---

## How to Reproduce

```bash
cargo run
```

The output shows the 9-week simulation and prints the unfair distribution.

## What Would Actually Catch This

1. **Property-based testing** with a fairness invariant:
   ```rust
   // "No person should have more than 2x the assignments of any other 
   // available person over any 4-week window"
   ```

2. **Statistical simulation testing** — run 1000 random availability patterns, assert max/min ratio < threshold.

3. **A human tester** who says "wait, Cara only took out the trash twice last month?"

Option 3 is what usually happens in practice.

---

## Running Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage (library only — main.rs is the demo)
cargo tarpaulin --skip-clean --exclude-files "src/main.rs" --out Stdout
```

Output:
```
|| src/lib.rs: 43/43
100.00% coverage, 43/43 lines covered
```
