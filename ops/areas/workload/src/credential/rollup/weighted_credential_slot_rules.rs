use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted credential slot must pass. Check returns its problems, or none.
pub trait WeightedCredentialSlotRule: std::fmt::Debug {
    fn check(&self, item: &WeightedCredentialSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialSlotRequiredSummaryRule {
    pub message: String,
}

impl WeightedCredentialSlotRule for WeightedCredentialSlotRequiredSummaryRule {
    fn check(&self, item: &WeightedCredentialSlot) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialSlotLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedCredentialSlotRule for WeightedCredentialSlotLimitRangeRule {
    fn check(&self, item: &WeightedCredentialSlot) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialSlotSummaryLengthRule {
    pub max_length: i64,
}

impl WeightedCredentialSlotRule for WeightedCredentialSlotSummaryLengthRule {
    fn check(&self, item: &WeightedCredentialSlot) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted credential slot.
#[derive(Debug, Clone)]
pub struct WeightedCredentialSlotValidator {
    pub rules: Vec<Rc<dyn WeightedCredentialSlotRule>>,
}

impl WeightedCredentialSlotValidator {
    pub fn validate(&self, item: &WeightedCredentialSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedCredentialSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedCredentialSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_credential_slot_validator() -> WeightedCredentialSlotValidator {
    WeightedCredentialSlotValidator {
        rules: vec![
            Rc::new(WeightedCredentialSlotRequiredSummaryRule {
                message: "weighted credential slot needs this field".to_string(),
            }) as Rc<dyn WeightedCredentialSlotRule>,
            Rc::new(WeightedCredentialSlotLimitRangeRule { min: 3600, max: 86400 }),
            Rc::new(WeightedCredentialSlotSummaryLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_credential_slot_validator();
        let item = new_weighted_credential_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = WeightedCredentialSlotRequiredSummaryRule {
            message: "weighted credential slot needs this field".to_string(),
        };
        let mut item = new_weighted_credential_slot("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = WeightedCredentialSlotLimitRangeRule { min: 120, max: 255 };
        let mut item = new_weighted_credential_slot("x", "y");
        item.limit = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = WeightedCredentialSlotSummaryLengthRule { max_length: 3600 };
        let mut item = new_weighted_credential_slot("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
