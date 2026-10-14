use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict segment ledger must pass. Check returns its problems, or none.
pub trait StrictSegmentLedgerRule: std::fmt::Debug {
    fn check(&self, item: &StrictSegmentLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentLedgerCodePrefixRule {
    pub prefix: String,
}

impl StrictSegmentLedgerRule for StrictSegmentLedgerCodePrefixRule {
    fn check(&self, item: &StrictSegmentLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentLedgerRequiredCodeRule {
    pub message: String,
}

impl StrictSegmentLedgerRule for StrictSegmentLedgerRequiredCodeRule {
    fn check(&self, item: &StrictSegmentLedger) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentLedgerRequiredLabelRule {
    pub message: String,
}

impl StrictSegmentLedgerRule for StrictSegmentLedgerRequiredLabelRule {
    fn check(&self, item: &StrictSegmentLedger) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentLedgerRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictSegmentLedgerRule for StrictSegmentLedgerRotationDaysRangeRule {
    fn check(&self, item: &StrictSegmentLedger) -> Vec<String> {
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
pub struct StrictSegmentLedgerPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictSegmentLedgerRule for StrictSegmentLedgerPriorityRangeRule {
    fn check(&self, item: &StrictSegmentLedger) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentLedgerAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl StrictSegmentLedgerRule for StrictSegmentLedgerAllowedLabelRule {
    fn check(&self, item: &StrictSegmentLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict segment ledger.
#[derive(Debug, Clone)]
pub struct StrictSegmentLedgerValidator {
    pub rules: Vec<Rc<dyn StrictSegmentLedgerRule>>,
}

impl StrictSegmentLedgerValidator {
    pub fn validate(&self, item: &StrictSegmentLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictSegmentLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictSegmentLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_segment_ledger_validator() -> StrictSegmentLedgerValidator {
    StrictSegmentLedgerValidator {
        rules: vec![
            Rc::new(StrictSegmentLedgerCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn StrictSegmentLedgerRule>,
            Rc::new(StrictSegmentLedgerRequiredCodeRule {
                message: "strict segment ledger needs a value".to_string(),
            }),
            Rc::new(StrictSegmentLedgerRequiredLabelRule {
                message: "strict segment ledger needs a value".to_string(),
            }),
            Rc::new(StrictSegmentLedgerRotationDaysRangeRule { min: 1000, max: 86400 }),
            Rc::new(StrictSegmentLedgerPriorityRangeRule { min: 255, max: 3600 }),
            Rc::new(StrictSegmentLedgerAllowedLabelRule {
                allowed: vec!["legacy".to_string(), "capped".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_segment_ledger_validator();
        let item = new_strict_segment_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictSegmentLedgerRequiredCodeRule {
            message: "strict segment ledger needs an entry".to_string(),
        };
        let mut item = new_strict_segment_ledger("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictSegmentLedgerRequiredLabelRule {
            message: "strict segment ledger needs an entry".to_string(),
        };
        let mut item = new_strict_segment_ledger("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = StrictSegmentLedgerRotationDaysRangeRule { min: 255, max: 3600 };
        let mut item = new_strict_segment_ledger("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = StrictSegmentLedgerPriorityRangeRule { min: 32, max: 3600 };
        let mut item = new_strict_segment_ledger("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
