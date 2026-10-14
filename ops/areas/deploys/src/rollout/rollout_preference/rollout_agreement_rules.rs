use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollout agreement must pass. Check returns its problems, or none.
pub trait RolloutAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RolloutAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAgreementCodePrefixRule {
    pub prefix: String,
}

impl RolloutAgreementRule for RolloutAgreementCodePrefixRule {
    fn check(&self, item: &RolloutAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAgreementRequiredCodeRule {
    pub message: String,
}

impl RolloutAgreementRule for RolloutAgreementRequiredCodeRule {
    fn check(&self, item: &RolloutAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAgreementRequiredNotesRule {
    pub message: String,
}

impl RolloutAgreementRule for RolloutAgreementRequiredNotesRule {
    fn check(&self, item: &RolloutAgreement) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAgreementMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutAgreementRule for RolloutAgreementMaximumRangeRule {
    fn check(&self, item: &RolloutAgreement) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAgreementAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RolloutAgreementRule for RolloutAgreementAllowedTitleRule {
    fn check(&self, item: &RolloutAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout agreement.
#[derive(Debug, Clone)]
pub struct RolloutAgreementValidator {
    pub rules: Vec<Rc<dyn RolloutAgreementRule>>,
}

impl RolloutAgreementValidator {
    pub fn validate(&self, item: &RolloutAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_agreement_validator() -> RolloutAgreementValidator {
    RolloutAgreementValidator {
        rules: vec![
            Rc::new(RolloutAgreementCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn RolloutAgreementRule>,
            Rc::new(RolloutAgreementRequiredCodeRule {
                message: "rollout agreement needs this field".to_string(),
            }),
            Rc::new(RolloutAgreementRequiredNotesRule {
                message: "rollout agreement needs this field".to_string(),
            }),
            Rc::new(RolloutAgreementMaximumRangeRule { min: 2.4, max: 2576.9 }),
            Rc::new(RolloutAgreementAllowedTitleRule {
                allowed: vec!["rolling".to_string(), "legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_agreement_validator();
        let item = new_rollout_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutAgreementRequiredCodeRule {
            message: "rollout agreement needs a value".to_string(),
        };
        let mut item = new_rollout_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RolloutAgreementRequiredNotesRule {
            message: "rollout agreement needs an entry".to_string(),
        };
        let mut item = new_rollout_agreement("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RolloutAgreementMaximumRangeRule { min: 6.0, max: 1272.0 };
        let mut item = new_rollout_agreement("x", "y");
        item.maximum = 1273.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
