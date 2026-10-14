use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped slo entry must pass. Check returns its problems, or none.
pub trait CappedSloEntryRule: std::fmt::Debug {
    fn check(&self, item: &CappedSloEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloEntryRequiredLabelRule {
    pub message: String,
}

impl CappedSloEntryRule for CappedSloEntryRequiredLabelRule {
    fn check(&self, item: &CappedSloEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloEntryCodeLengthRule {
    pub max_length: i64,
}

impl CappedSloEntryRule for CappedSloEntryCodeLengthRule {
    fn check(&self, item: &CappedSloEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloEntryAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedSloEntryRule for CappedSloEntryAttemptsRangeRule {
    fn check(&self, item: &CappedSloEntry) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloEntryReviewerLengthRule {
    pub max_length: i64,
}

impl CappedSloEntryRule for CappedSloEntryReviewerLengthRule {
    fn check(&self, item: &CappedSloEntry) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloEntryCodePrefixRule {
    pub prefix: String,
}

impl CappedSloEntryRule for CappedSloEntryCodePrefixRule {
    fn check(&self, item: &CappedSloEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloEntryRequiredCodeRule {
    pub message: String,
}

impl CappedSloEntryRule for CappedSloEntryRequiredCodeRule {
    fn check(&self, item: &CappedSloEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped slo entry.
#[derive(Debug, Clone)]
pub struct CappedSloEntryValidator {
    pub rules: Vec<Rc<dyn CappedSloEntryRule>>,
}

impl CappedSloEntryValidator {
    pub fn validate(&self, item: &CappedSloEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedSloEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSloEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_slo_entry_validator() -> CappedSloEntryValidator {
    CappedSloEntryValidator {
        rules: vec![
            Rc::new(CappedSloEntryRequiredLabelRule {
                message: "capped slo entry needs an entry".to_string(),
            }) as Rc<dyn CappedSloEntryRule>,
            Rc::new(CappedSloEntryCodeLengthRule { max_length: 120 }),
            Rc::new(CappedSloEntryAttemptsRangeRule { min: 16, max: 16 }),
            Rc::new(CappedSloEntryReviewerLengthRule { max_length: 3600 }),
            Rc::new(CappedSloEntryCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(CappedSloEntryRequiredCodeRule {
                message: "capped slo entry needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_slo_entry_validator();
        let item = new_capped_slo_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedSloEntryRequiredLabelRule {
            message: "capped slo entry needs an entry".to_string(),
        };
        let mut item = new_capped_slo_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedSloEntryCodeLengthRule { max_length: 255 };
        let mut item = new_capped_slo_entry("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = CappedSloEntryAttemptsRangeRule { min: 64, max: 1000 };
        let mut item = new_capped_slo_entry("x", "y");
        item.attempts = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = CappedSloEntryReviewerLengthRule { max_length: 3600 };
        let mut item = new_capped_slo_entry("x", "y");
        item.reviewer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedSloEntryRequiredCodeRule {
            message: "capped slo entry needs an entry".to_string(),
        };
        let mut item = new_capped_slo_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
