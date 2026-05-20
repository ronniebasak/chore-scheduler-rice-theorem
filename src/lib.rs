use std::collections::{HashMap, VecDeque};

/// Represents a roommate in the household.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Person {
    pub name: String,
}

impl Person {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

/// A record of who was assigned what chore.
#[derive(Debug, Clone)]
pub struct Assignment {
    pub person: Person,
    pub chore: String,
}

/// A fair chore rotation scheduler.
///
/// Fairness rules:
/// 1. Nobody should repeat the same chore consecutively.
/// 2. People with fewer completed chores should be prioritized.
/// 3. Recently assigned users get a temporary cooldown (2 rounds).
/// 4. Users may be unavailable on certain days.
#[derive(Debug)]
pub struct Scheduler {
    history: VecDeque<Assignment>,
    completed: HashMap<Person, usize>,
    cooldowns: HashMap<Person, usize>,
    unavailable: HashMap<Person, Vec<String>>,
}

impl Scheduler {
    /// Create a new scheduler with no history.
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            completed: HashMap::new(),
            cooldowns: HashMap::new(),
            unavailable: HashMap::new(),
        }
    }

    /// Mark a person as unavailable on a given day.
    pub fn mark_unavailable(&mut self, person: &Person, day: &str) {
        self.unavailable
            .entry(person.clone())
            .or_default()
            .push(day.to_string());
    }

    /// Get the total number of assignments completed by each person.
    pub fn get_completed(&self) -> &HashMap<Person, usize> {
        &self.completed
    }

    /// Get the current cooldown values for each person.
    pub fn get_cooldowns(&self) -> &HashMap<Person, usize> {
        &self.cooldowns
    }

    /// Get the full assignment history.
    pub fn get_history(&self) -> &VecDeque<Assignment> {
        &self.history
    }

    /// Assign a chore to the fairest candidate for a given day.
    ///
    /// Returns `None` if no eligible candidate exists (e.g., everyone is
    /// unavailable or the only candidates would repeat the same chore).
    pub fn assign(
        &mut self,
        people: &[Person],
        chore: &str,
        day: &str,
    ) -> Option<Person> {
        // Find who last did this specific chore (to avoid consecutive repeats).
        let last_person_for_chore = self
            .history
            .iter()
            .rev()
            .find(|a| a.chore == chore)
            .map(|a| a.person.clone());

        // Filter out unavailable people for this day.
        let mut candidates: Vec<Person> = people
            .iter()
            .filter(|p| {
                !self
                    .unavailable
                    .get(*p)
                    .map(|days| days.contains(&day.to_string()))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        // Sort by fairness criteria:
        // Primary: lower cooldown first (recently assigned people wait).
        // Secondary: fewer completed chores first (balance workload).
        candidates.sort_by_key(|p| {
            (
                self.cooldowns.get(p).copied().unwrap_or(0),
                self.completed.get(p).copied().unwrap_or(0),
            )
        });

        // Pick the first candidate who didn't just do this same chore.
        let selected = candidates
            .into_iter()
            .find(|p| Some(p.clone()) != last_person_for_chore);

        if let Some(ref person) = selected {
            // Record the assignment.
            *self.completed.entry(person.clone()).or_insert(0) += 1;

            // Decay all existing cooldowns by 1.
            self.cooldowns
                .values_mut()
                .for_each(|v| *v = v.saturating_sub(1));

            // Set the assigned person's cooldown to 2.
            self.cooldowns.insert(person.clone(), 2);

            // Add to history, keeping a bounded window.
            self.history.push_back(Assignment {
                person: person.clone(),
                chore: chore.to_string(),
            });

            if self.history.len() > 30 {
                self.history.pop_front();
            }
        }

        selected
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
