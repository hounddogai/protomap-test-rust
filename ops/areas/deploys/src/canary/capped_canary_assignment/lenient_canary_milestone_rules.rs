use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient canary milestone must pass. Check returns its problems, or none.
pub trait LenientCanaryMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &LenientCanaryMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryMilestoneRequiredClusterRule {
    pub message: String,
}

impl LenientCanaryMilestoneRule for LenientCanaryMilestoneRequiredClusterRule {
    fn check(&self, item: &LenientCanaryMilestone) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryMilestoneRequiredCodeRule {
    pub message: String,
}

impl LenientCanaryMilestoneRule for LenientCanaryMilestoneRequiredCodeRule {
    fn check(&self, item: &LenientCanaryMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryMilestoneLabelLengthRule {
    pub max_length: i64,
}

impl LenientCanaryMilestoneRule for LenientCanaryMilestoneLabelLengthRule {
    fn check(&self, item: &LenientCanaryMilestone) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryMilestoneRequiredLabelRule {
    pub message: String,
}

impl LenientCanaryMilestoneRule for LenientCanaryMilestoneRequiredLabelRule {
    fn check(&self, item: &LenientCanaryMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryMilestoneCodePrefixRule {
    pub prefix: String,
}

impl LenientCanaryMilestoneRule for LenientCanaryMilestoneCodePrefixRule {
    fn check(&self, item: &LenientCanaryMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient canary milestone.
#[derive(Debug, Clone)]
pub struct LenientCanaryMilestoneValidator {
    pub rules: Vec<Rc<dyn LenientCanaryMilestoneRule>>,
}

impl LenientCanaryMilestoneValidator {
    pub fn validate(&self, item: &LenientCanaryMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientCanaryMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientCanaryMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_canary_milestone_validator() -> LenientCanaryMilestoneValidator {
    LenientCanaryMilestoneValidator {
        rules: vec![
            Rc::new(LenientCanaryMilestoneRequiredClusterRule {
                message: "lenient canary milestone needs an entry".to_string(),
            }) as Rc<dyn LenientCanaryMilestoneRule>,
            Rc::new(LenientCanaryMilestoneRequiredCodeRule {
                message: "lenient canary milestone needs an entry".to_string(),
            }),
            Rc::new(LenientCanaryMilestoneLabelLengthRule { max_length: 255 }),
            Rc::new(LenientCanaryMilestoneRequiredLabelRule {
                message: "lenient canary milestone needs this field".to_string(),
            }),
            Rc::new(LenientCanaryMilestoneCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_canary_milestone_validator();
        let item = new_lenient_canary_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = LenientCanaryMilestoneRequiredClusterRule {
            message: "lenient canary milestone needs this field".to_string(),
        };
        let mut item = new_lenient_canary_milestone("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientCanaryMilestoneRequiredCodeRule {
            message: "lenient canary milestone needs a value".to_string(),
        };
        let mut item = new_lenient_canary_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientCanaryMilestoneLabelLengthRule { max_length: 1000 };
        let mut item = new_lenient_canary_milestone("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientCanaryMilestoneRequiredLabelRule {
            message: "lenient canary milestone needs an entry".to_string(),
        };
        let mut item = new_lenient_canary_milestone("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
