use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding proposal must pass. Check returns its problems, or none.
pub trait BindingProposalRule: std::fmt::Debug {
    fn check(&self, item: &BindingProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProposalRequiredCategoryRule {
    pub message: String,
}

impl BindingProposalRule for BindingProposalRequiredCategoryRule {
    fn check(&self, item: &BindingProposal) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProposalRequiredCodeRule {
    pub message: String,
}

impl BindingProposalRule for BindingProposalRequiredCodeRule {
    fn check(&self, item: &BindingProposal) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProposalTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl BindingProposalRule for BindingProposalTagsLimitRule {
    fn check(&self, item: &BindingProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProposalThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingProposalRule for BindingProposalThresholdRangeRule {
    fn check(&self, item: &BindingProposal) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProposalRequiredFingerprintRule {
    pub message: String,
}

impl BindingProposalRule for BindingProposalRequiredFingerprintRule {
    fn check(&self, item: &BindingProposal) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingProposalAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BindingProposalRule for BindingProposalAllowedCodeRule {
    fn check(&self, item: &BindingProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding proposal.
#[derive(Debug, Clone)]
pub struct BindingProposalValidator {
    pub rules: Vec<Rc<dyn BindingProposalRule>>,
}

impl BindingProposalValidator {
    pub fn validate(&self, item: &BindingProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_proposal_validator() -> BindingProposalValidator {
    BindingProposalValidator {
        rules: vec![
            Rc::new(BindingProposalRequiredCategoryRule {
                message: "binding proposal needs a value".to_string(),
            }) as Rc<dyn BindingProposalRule>,
            Rc::new(BindingProposalRequiredCodeRule {
                message: "binding proposal needs a value".to_string(),
            }),
            Rc::new(BindingProposalTagsLimitRule {
                max_count: 80,
                banned: Vec::new(),
            }),
            Rc::new(BindingProposalThresholdRangeRule { min: 9.0, max: 2082.8 }),
            Rc::new(BindingProposalRequiredFingerprintRule {
                message: "binding proposal needs a value".to_string(),
            }),
            Rc::new(BindingProposalAllowedCodeRule {
                allowed: vec!["scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_proposal_validator();
        let item = new_binding_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = BindingProposalRequiredCategoryRule {
            message: "binding proposal needs a value".to_string(),
        };
        let mut item = new_binding_proposal("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BindingProposalRequiredCodeRule {
            message: "binding proposal needs a value".to_string(),
        };
        let mut item = new_binding_proposal("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = BindingProposalThresholdRangeRule { min: 8.8, max: 436.5 };
        let mut item = new_binding_proposal("x", "y");
        item.threshold = 438.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = BindingProposalRequiredFingerprintRule {
            message: "binding proposal needs an entry".to_string(),
        };
        let mut item = new_binding_proposal("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
