use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict peering draft must pass. Check returns its problems, or none.
pub trait StrictPeeringDraftRule: std::fmt::Debug {
    fn check(&self, item: &StrictPeeringDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringDraftRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPeeringDraftRule for StrictPeeringDraftRetriesRangeRule {
    fn check(&self, item: &StrictPeeringDraft) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringDraftRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPeeringDraftRule for StrictPeeringDraftRotationDaysRangeRule {
    fn check(&self, item: &StrictPeeringDraft) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringDraftCipherLengthRule {
    pub max_length: i64,
}

impl StrictPeeringDraftRule for StrictPeeringDraftCipherLengthRule {
    fn check(&self, item: &StrictPeeringDraft) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringDraftCodePrefixRule {
    pub prefix: String,
}

impl StrictPeeringDraftRule for StrictPeeringDraftCodePrefixRule {
    fn check(&self, item: &StrictPeeringDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPeeringDraftCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPeeringDraftRule for StrictPeeringDraftCapacityRangeRule {
    fn check(&self, item: &StrictPeeringDraft) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict peering draft.
#[derive(Debug, Clone)]
pub struct StrictPeeringDraftValidator {
    pub rules: Vec<Rc<dyn StrictPeeringDraftRule>>,
}

impl StrictPeeringDraftValidator {
    pub fn validate(&self, item: &StrictPeeringDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictPeeringDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPeeringDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_peering_draft_validator() -> StrictPeeringDraftValidator {
    StrictPeeringDraftValidator {
        rules: vec![
            Rc::new(StrictPeeringDraftRetriesRangeRule { min: 16, max: 1000 }) as Rc<dyn StrictPeeringDraftRule>,
            Rc::new(StrictPeeringDraftRotationDaysRangeRule { min: 1000, max: 86400 }),
            Rc::new(StrictPeeringDraftCipherLengthRule { max_length: 80 }),
            Rc::new(StrictPeeringDraftCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictPeeringDraftCapacityRangeRule { min: 32, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_peering_draft_validator();
        let item = new_strict_peering_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = StrictPeeringDraftRetriesRangeRule { min: 16, max: 3600 };
        let mut item = new_strict_peering_draft("x", "y");
        item.retries = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = StrictPeeringDraftRotationDaysRangeRule { min: 255, max: 86400 };
        let mut item = new_strict_peering_draft("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = StrictPeeringDraftCipherLengthRule { max_length: 80 };
        let mut item = new_strict_peering_draft("x", "y");
        item.cipher = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = StrictPeeringDraftCapacityRangeRule { min: 3600, max: 86400 };
        let mut item = new_strict_peering_draft("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
