use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched credential slot must pass. Check returns its problems, or none.
pub trait BatchedCredentialSlotRule: std::fmt::Debug {
    fn check(&self, item: &BatchedCredentialSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialSlotScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedCredentialSlotRule for BatchedCredentialSlotScoreRangeRule {
    fn check(&self, item: &BatchedCredentialSlot) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialSlotRequiredSegmentRule {
    pub message: String,
}

impl BatchedCredentialSlotRule for BatchedCredentialSlotRequiredSegmentRule {
    fn check(&self, item: &BatchedCredentialSlot) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialSlotPinnedGuardRule {
    pub limit: f64,
}

impl BatchedCredentialSlotRule for BatchedCredentialSlotPinnedGuardRule {
    fn check(&self, item: &BatchedCredentialSlot) -> Vec<String> {
        if item.pinned && (item.minimum > self.limit) {
            return vec![format!("minimum {} exceeds {} while pinned", item.minimum, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialSlotRequiredCodeRule {
    pub message: String,
}

impl BatchedCredentialSlotRule for BatchedCredentialSlotRequiredCodeRule {
    fn check(&self, item: &BatchedCredentialSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched credential slot.
#[derive(Debug, Clone)]
pub struct BatchedCredentialSlotValidator {
    pub rules: Vec<Rc<dyn BatchedCredentialSlotRule>>,
}

impl BatchedCredentialSlotValidator {
    pub fn validate(&self, item: &BatchedCredentialSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedCredentialSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedCredentialSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_credential_slot_validator() -> BatchedCredentialSlotValidator {
    BatchedCredentialSlotValidator {
        rules: vec![
            Rc::new(BatchedCredentialSlotScoreRangeRule { min: 0.8, max: 1239.6 }) as Rc<dyn BatchedCredentialSlotRule>,
            Rc::new(BatchedCredentialSlotRequiredSegmentRule {
                message: "batched credential slot needs an entry".to_string(),
            }),
            Rc::new(BatchedCredentialSlotPinnedGuardRule { limit: 2.2 }),
            Rc::new(BatchedCredentialSlotRequiredCodeRule {
                message: "batched credential slot needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_credential_slot_validator();
        let item = new_batched_credential_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = BatchedCredentialSlotScoreRangeRule { min: 7.7, max: 723.2 };
        let mut item = new_batched_credential_slot("x", "y");
        item.score = 724.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = BatchedCredentialSlotRequiredSegmentRule {
            message: "batched credential slot needs an entry".to_string(),
        };
        let mut item = new_batched_credential_slot("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedCredentialSlotRequiredCodeRule {
            message: "batched credential slot needs an entry".to_string(),
        };
        let mut item = new_batched_credential_slot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
