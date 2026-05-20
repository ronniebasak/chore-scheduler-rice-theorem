use chore_scheduler::{Person, Scheduler};

fn p(name: &str) -> Person {
    Person::new(name)
}

// ─── Basic Assignment ──────────────────────────────────────────────────────────

#[test]
fn assigns_someone_when_candidates_exist() {
    let mut s = Scheduler::new();
    let people = vec![p("Alice"), p("Bob")];

    let result = s.assign(&people, "dishes", "monday");

    assert!(result.is_some());
}

#[test]
fn returns_none_when_all_unavailable() {
    let mut s = Scheduler::new();
    let alice = p("Alice");

    s.mark_unavailable(&alice, "monday");

    let result = s.assign(&[alice], "dishes", "monday");

    assert!(result.is_none());
}

// ─── No Consecutive Repeats ────────────────────────────────────────────────────

#[test]
fn never_assigns_same_person_same_chore_consecutively() {
    let mut s = Scheduler::new();
    let people = vec![p("Alice"), p("Bob"), p("Cara")];

    let first = s.assign(&people, "trash", "monday").unwrap();
    let second = s.assign(&people, "trash", "tuesday").unwrap();

    assert_ne!(first, second);
}

#[test]
fn allows_same_person_for_different_chores() {
    let mut s = Scheduler::new();
    let people = vec![p("Alice")];

    let first = s.assign(&people, "dishes", "monday").unwrap();
    let second = s.assign(&people, "trash", "tuesday").unwrap();

    // Same person is fine if it's a different chore.
    assert_eq!(first, second);
}

// ─── Unavailability ────────────────────────────────────────────────────────────

#[test]
fn respects_unavailability() {
    let mut s = Scheduler::new();
    let alice = p("Alice");
    let bob = p("Bob");

    s.mark_unavailable(&alice, "monday");

    let result = s.assign(&[alice.clone(), bob.clone()], "laundry", "monday");

    assert_eq!(result.unwrap(), bob);
}

#[test]
fn unavailability_is_day_specific() {
    let mut s = Scheduler::new();
    let alice = p("Alice");

    s.mark_unavailable(&alice, "monday");

    // Alice is available on tuesday.
    let result = s.assign(&[alice.clone()], "dishes", "tuesday");
    assert_eq!(result.unwrap(), alice);
}

// ─── Cooldown Mechanics ────────────────────────────────────────────────────────

#[test]
fn cooldowns_decay_each_round() {
    let mut s = Scheduler::new();
    let people = vec![p("Alice"), p("Bob"), p("Cara")];

    s.assign(&people, "dishes", "monday");
    s.assign(&people, "trash", "tuesday");
    s.assign(&people, "laundry", "wednesday");

    for value in s.get_cooldowns().values() {
        assert!(*value <= 2);
    }
}

#[test]
fn recently_assigned_person_gets_cooldown() {
    let mut s = Scheduler::new();
    let people = vec![p("Alice"), p("Bob")];

    let first = s.assign(&people, "dishes", "monday").unwrap();

    // The person who was just assigned should have a cooldown of 2.
    assert_eq!(s.get_cooldowns().get(&first).copied().unwrap_or(0), 2);
}

#[test]
fn cooldown_prevents_immediate_reassignment() {
    let mut s = Scheduler::new();
    let people = vec![p("Alice"), p("Bob"), p("Cara")];

    let first = s.assign(&people, "dishes", "monday").unwrap();
    let second = s.assign(&people, "trash", "tuesday").unwrap();

    // With 3 people and cooldown=2, the third assignment should go
    // to the remaining person (not first or second).
    let third = s.assign(&people, "sweeping", "wednesday").unwrap();

    assert_ne!(third, first);
    assert_ne!(third, second);
}

// ─── Workload Balancing ────────────────────────────────────────────────────────

#[test]
fn prioritizes_person_with_fewer_assignments() {
    let mut s = Scheduler::new();
    let alice = p("Alice");
    let bob = p("Bob");

    // Give Alice multiple assignments first.
    s.assign(&[alice.clone(), bob.clone()], "dishes", "monday");
    s.assign(&[alice.clone(), bob.clone()], "trash", "tuesday");

    // After cooldowns decay and assignments accumulate,
    // the system should lean toward the person with fewer total.
    // Assign several more rounds to see balancing.
    let people = vec![alice.clone(), bob.clone()];
    for i in 0..6 {
        let day = format!("day{}", i + 3);
        let chore = if i % 2 == 0 { "dishes" } else { "trash" };
        s.assign(&people, chore, &day);
    }

    let completed = s.get_completed();
    let alice_count = completed.get(&alice).copied().unwrap_or(0);
    let bob_count = completed.get(&bob).copied().unwrap_or(0);

    // With 2 people over 8 rounds, neither should have more than 5
    // (perfect split is 4 each).
    assert!(alice_count <= 5);
    assert!(bob_count <= 5);
}

// ─── History Bounds ────────────────────────────────────────────────────────────

#[test]
fn history_stays_bounded() {
    let mut s = Scheduler::new();
    let people = vec![p("Alice"), p("Bob"), p("Cara")];

    // Make 50 assignments — history should be capped at 30.
    for i in 0..50 {
        let day = format!("day{}", i);
        let chore = ["dishes", "trash", "sweeping"][i % 3];
        s.assign(&people, chore, &day);
    }

    assert!(s.get_history().len() <= 30);
}

// ─── Edge Cases ────────────────────────────────────────────────────────────────

#[test]
fn single_person_rotates_between_chores() {
    let mut s = Scheduler::new();
    let alice = p("Alice");

    // With only one person, they should get assigned to different chores
    // but never the same chore twice in a row (returns None if stuck).
    let r1 = s.assign(&[alice.clone()], "dishes", "monday");
    assert_eq!(r1.unwrap(), alice);

    let r2 = s.assign(&[alice.clone()], "dishes", "tuesday");
    // Can't assign Alice to dishes again consecutively — no other candidate.
    assert!(r2.is_none());

    let r3 = s.assign(&[alice.clone()], "trash", "wednesday");
    // Different chore is fine.
    assert_eq!(r3.unwrap(), alice);
}

#[test]
fn empty_people_list_returns_none() {
    let mut s = Scheduler::new();
    let result = s.assign(&[], "dishes", "monday");
    assert!(result.is_none());
}

#[test]
fn multiple_unavailable_days_for_same_person() {
    let mut s = Scheduler::new();
    let alice = p("Alice");
    let bob = p("Bob");

    s.mark_unavailable(&alice, "monday");
    s.mark_unavailable(&alice, "tuesday");
    s.mark_unavailable(&alice, "wednesday");

    // Alice should still be available on thursday.
    let result = s.assign(&[alice.clone(), bob.clone()], "dishes", "thursday");
    // Alice has 0 completed and 0 cooldown, so she should be picked.
    assert_eq!(result.unwrap(), alice);
}

#[test]
fn default_trait_works() {
    let s = Scheduler::default();
    assert!(s.get_history().is_empty());
    assert!(s.get_completed().is_empty());
    assert!(s.get_cooldowns().is_empty());
}
