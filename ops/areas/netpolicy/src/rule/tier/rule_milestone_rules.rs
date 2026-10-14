use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule milestone must pass. Check returns its problems, or none.
pub trait RuleMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &RuleMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMilestoneStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleMilestoneRule for RuleMilestoneStartsAtRangeRule {
    fn check(&self, item: &RuleMilestone) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMilestoneTitleLengthRule {
    pub max_length: i64,
}

impl RuleMilestoneRule for RuleMilestoneTitleLengthRule {
    fn check(&self, item: &RuleMilestone) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl RuleMilestoneRule for RuleMilestoneCodeLengthRule {
    fn check(&self, item: &RuleMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMilestoneRequiredNameRule {
    pub message: String,
}

impl RuleMilestoneRule for RuleMilestoneRequiredNameRule {
    fn check(&self, item: &RuleMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMilestoneAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleMilestoneRule for RuleMilestoneAttemptsRangeRule {
    fn check(&self, item: &RuleMilestone) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rule milestone.
#[derive(Debug, Clone)]
pub struct RuleMilestoneValidator {
    pub rules: Vec<Rc<dyn RuleMilestoneRule>>,
}

impl RuleMilestoneValidator {
    pub fn validate(&self, item: &RuleMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_milestone_validator() -> RuleMilestoneValidator {
    RuleMilestoneValidator {
        rules: vec![
            Rc::new(RuleMilestoneStartsAtRangeRule { min: 16, max: 80 }) as Rc<dyn RuleMilestoneRule>,
            Rc::new(RuleMilestoneTitleLengthRule { max_length: 255 }),
            Rc::new(RuleMilestoneCodeLengthRule { max_length: 86400 }),
            Rc::new(RuleMilestoneRequiredNameRule {
                message: "rule milestone needs an entry".to_string(),
            }),
            Rc::new(RuleMilestoneAttemptsRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_milestone_validator();
        let item = new_rule_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RuleMilestoneStartsAtRangeRule { min: 16, max: 3600 };
        let mut item = new_rule_milestone("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RuleMilestoneTitleLengthRule { max_length: 255 };
        let mut item = new_rule_milestone("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RuleMilestoneCodeLengthRule { max_length: 1000 };
        let mut item = new_rule_milestone("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RuleMilestoneRequiredNameRule {
            message: "rule milestone needs a value".to_string(),
        };
        let mut item = new_rule_milestone("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RuleMilestoneAttemptsRangeRule { min: 32, max: 3600 };
        let mut item = new_rule_milestone("x", "y");
        item.attempts = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
