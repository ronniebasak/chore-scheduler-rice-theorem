use chore_scheduler::{Person, Scheduler};

fn main() {
    let alice = Person::new("Alice");
    let bob = Person::new("Bob");
    let cara = Person::new("Cara");

    let people = vec![alice.clone(), bob.clone(), cara.clone()];
    let chores = ["dishes", "trash", "sweeping"];
    let days = [
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
    ];

    let mut scheduler = Scheduler::new();

    // Realistic scenario: Cara is frequently unavailable on weekdays
    // (she works late shifts on Mon/Tue/Wed).
    scheduler.mark_unavailable(&cara, "monday");
    scheduler.mark_unavailable(&cara, "tuesday");
    scheduler.mark_unavailable(&cara, "wednesday");

    println!("=== Chore Scheduler: 9-Week Simulation ===");
    println!("Rules:");
    println!("  1. No one repeats the same chore consecutively");
    println!("  2. Fewer total chores → higher priority");
    println!("  3. Recently assigned people get a 2-round cooldown");
    println!("  4. Cara is unavailable Mon/Tue/Wed (late shifts)\n");

    // Simulate 9 weeks of chore assignments.
    let mut week_num = 0;

    for day_index in 0..63 {
        let day = days[day_index % 7];
        let chore = chores[day_index % 3];

        if day_index % 7 == 0 {
            week_num += 1;
            println!("--- Week {} ---", week_num);
        }

        let assigned = scheduler.assign(&people, chore, day);

        match assigned {
            Some(ref person) => {
                println!("  {} ({}):  → {}", day, chore, person.name);
            }
            None => {
                println!("  {} ({}): no one available!", day, chore);
            }
        }
    }

    // Print final workload distribution.
    println!("\n=== Final Workload Distribution ===");
    let completed = scheduler.get_completed();

    let mut totals: Vec<(&Person, usize)> = people
        .iter()
        .map(|p| (p, completed.get(p).copied().unwrap_or(0)))
        .collect();
    totals.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    let total_assignments: usize = totals.iter().map(|(_, c)| c).sum();

    for (person, count) in &totals {
        let pct = (*count as f64 / total_assignments as f64) * 100.0;
        println!("  {}: {} assignments ({:.1}%)", person.name, count, pct);
    }

    let max = totals.first().map(|(_, c)| *c).unwrap_or(0);
    let min = totals.last().map(|(_, c)| *c).unwrap_or(0);
    let ideal = total_assignments as f64 / people.len() as f64;

    println!("\n  Ideal (perfectly fair): {:.1} each", ideal);
    println!("  Actual spread: {} to {} (delta: {})", min, max, max - min);

    if max - min > 5 {
        println!(
            "\n  ⚠️  UNFAIR: The spread of {} exceeds reasonable bounds.",
            max - min
        );
        println!("  One roommate is doing significantly more than their fair share.");
        println!("  All tests pass. Coverage is 100%. The scheduler is 'correct'.");
    } else {
        println!("\n  ✓ Distribution looks reasonably fair.");
    }
}
