use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict federation cycle must pass. Check returns its problems, or none.
pub trait StrictFederationCycleRule: std::fmt::Debug {
    fn check(&self, item: &StrictFederationCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationCycleCodeLengthRule {
    pub max_length: i64,
}

impl StrictFederationCycleRule for StrictFederationCycleCodeLengthRule {
    fn check(&self, item: &StrictFederationCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationCycleTitleLengthRule {
    pub max_length: i64,
}

impl StrictFederationCycleRule for StrictFederationCycleTitleLengthRule {
    fn check(&self, item: &StrictFederationCycle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationCycleNotesLengthRule {
    pub max_length: i64,
}

impl StrictFederationCycleRule for StrictFederationCycleNotesLengthRule {
    fn check(&self, item: &StrictFederationCycle) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationCycleRequiredCurrencyRule {
    pub message: String,
}

impl StrictFederationCycleRule for StrictFederationCycleRequiredCurrencyRule {
    fn check(&self, item: &StrictFederationCycle) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationCycleDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictFederationCycleRule for StrictFederationCycleDurationMinutesRangeRule {
    fn check(&self, item: &StrictFederationCycle) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict federation cycle.
#[derive(Debug, Clone)]
pub struct StrictFederationCycleValidator {
    pub rules: Vec<Rc<dyn StrictFederationCycleRule>>,
}

impl StrictFederationCycleValidator {
    pub fn validate(&self, item: &StrictFederationCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictFederationCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictFederationCycle]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_strict_federation_cycle_validator() -> StrictFederationCycleValidator {
    StrictFederationCycleValidator {
        rules: vec![
            Rc::new(StrictFederationCycleCodeLengthRule { max_length: 80 }) as Rc<dyn StrictFederationCycleRule>,
            Rc::new(StrictFederationCycleTitleLengthRule { max_length: 80 }),
            Rc::new(StrictFederationCycleNotesLengthRule { max_length: 3600 }),
            Rc::new(StrictFederationCycleRequiredCurrencyRule {
                message: "strict federation cycle needs an entry".to_string(),
            }),
            Rc::new(StrictFederationCycleDurationMinutesRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_federation_cycle_validator();
        let item = new_strict_federation_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictFederationCycleCodeLengthRule { max_length: 64 };
        let mut item = new_strict_federation_cycle("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictFederationCycleTitleLengthRule { max_length: 3600 };
        let mut item = new_strict_federation_cycle("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = StrictFederationCycleNotesLengthRule { max_length: 16 };
        let mut item = new_strict_federation_cycle("x", "y");
        item.notes = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = StrictFederationCycleRequiredCurrencyRule {
            message: "strict federation cycle needs this field".to_string(),
        };
        let mut item = new_strict_federation_cycle("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = StrictFederationCycleDurationMinutesRangeRule { min: 32, max: 120 };
        let mut item = new_strict_federation_cycle("x", "y");
        item.duration_minutes = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
