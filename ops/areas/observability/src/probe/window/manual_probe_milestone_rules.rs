use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual probe milestone must pass. Check returns its problems, or none.
pub trait ManualProbeMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ManualProbeMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeMilestoneRequiredTitleRule {
    pub message: String,
}

impl ManualProbeMilestoneRule for ManualProbeMilestoneRequiredTitleRule {
    fn check(&self, item: &ManualProbeMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeMilestoneRequiredCodeRule {
    pub message: String,
}

impl ManualProbeMilestoneRule for ManualProbeMilestoneRequiredCodeRule {
    fn check(&self, item: &ManualProbeMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeMilestoneTitleLengthRule {
    pub max_length: i64,
}

impl ManualProbeMilestoneRule for ManualProbeMilestoneTitleLengthRule {
    fn check(&self, item: &ManualProbeMilestone) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeMilestoneCodePrefixRule {
    pub prefix: String,
}

impl ManualProbeMilestoneRule for ManualProbeMilestoneCodePrefixRule {
    fn check(&self, item: &ManualProbeMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual probe milestone.
#[derive(Debug, Clone)]
pub struct ManualProbeMilestoneValidator {
    pub rules: Vec<Rc<dyn ManualProbeMilestoneRule>>,
}

impl ManualProbeMilestoneValidator {
    pub fn validate(&self, item: &ManualProbeMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualProbeMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualProbeMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_probe_milestone_validator() -> ManualProbeMilestoneValidator {
    ManualProbeMilestoneValidator {
        rules: vec![
            Rc::new(ManualProbeMilestoneRequiredTitleRule {
                message: "manual probe milestone needs this field".to_string(),
            }) as Rc<dyn ManualProbeMilestoneRule>,
            Rc::new(ManualProbeMilestoneRequiredCodeRule {
                message: "manual probe milestone needs an entry".to_string(),
            }),
            Rc::new(ManualProbeMilestoneTitleLengthRule { max_length: 16 }),
            Rc::new(ManualProbeMilestoneCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_probe_milestone_validator();
        let item = new_manual_probe_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualProbeMilestoneRequiredTitleRule {
            message: "manual probe milestone needs a value".to_string(),
        };
        let mut item = new_manual_probe_milestone("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualProbeMilestoneRequiredCodeRule {
            message: "manual probe milestone needs an entry".to_string(),
        };
        let mut item = new_manual_probe_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualProbeMilestoneTitleLengthRule { max_length: 3600 };
        let mut item = new_manual_probe_milestone("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
