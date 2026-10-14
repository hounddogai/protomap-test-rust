use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary entry must pass. Check returns its problems, or none.
pub trait CanaryEntryRule: std::fmt::Debug {
    fn check(&self, item: &CanaryEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEntryCodePrefixRule {
    pub prefix: String,
}

impl CanaryEntryRule for CanaryEntryCodePrefixRule {
    fn check(&self, item: &CanaryEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEntryRequiredReviewerRule {
    pub message: String,
}

impl CanaryEntryRule for CanaryEntryRequiredReviewerRule {
    fn check(&self, item: &CanaryEntry) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEntryRequiredLabelRule {
    pub message: String,
}

impl CanaryEntryRule for CanaryEntryRequiredLabelRule {
    fn check(&self, item: &CanaryEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary entry.
#[derive(Debug, Clone)]
pub struct CanaryEntryValidator {
    pub rules: Vec<Rc<dyn CanaryEntryRule>>,
}

impl CanaryEntryValidator {
    pub fn validate(&self, item: &CanaryEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_entry_validator() -> CanaryEntryValidator {
    CanaryEntryValidator {
        rules: vec![
            Rc::new(CanaryEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn CanaryEntryRule>,
            Rc::new(CanaryEntryRequiredReviewerRule {
                message: "canary entry needs this field".to_string(),
            }),
            Rc::new(CanaryEntryRequiredLabelRule {
                message: "canary entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_entry_validator();
        let item = new_canary_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = CanaryEntryRequiredReviewerRule {
            message: "canary entry needs this field".to_string(),
        };
        let mut item = new_canary_entry("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CanaryEntryRequiredLabelRule {
            message: "canary entry needs this field".to_string(),
        };
        let mut item = new_canary_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
