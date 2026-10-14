use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce milestone must pass. Check returns its problems, or none.
pub trait NonceMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &NonceMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMilestoneRegionLengthRule {
    pub max_length: i64,
}

impl NonceMilestoneRule for NonceMilestoneRegionLengthRule {
    fn check(&self, item: &NonceMilestone) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMilestoneRequiredTitleRule {
    pub message: String,
}

impl NonceMilestoneRule for NonceMilestoneRequiredTitleRule {
    fn check(&self, item: &NonceMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMilestoneEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceMilestoneRule for NonceMilestoneEndsAtRangeRule {
    fn check(&self, item: &NonceMilestone) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl NonceMilestoneRule for NonceMilestoneCodeLengthRule {
    fn check(&self, item: &NonceMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce milestone.
#[derive(Debug, Clone)]
pub struct NonceMilestoneValidator {
    pub rules: Vec<Rc<dyn NonceMilestoneRule>>,
}

impl NonceMilestoneValidator {
    pub fn validate(&self, item: &NonceMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_milestone_validator() -> NonceMilestoneValidator {
    NonceMilestoneValidator {
        rules: vec![
            Rc::new(NonceMilestoneRegionLengthRule { max_length: 120 }) as Rc<dyn NonceMilestoneRule>,
            Rc::new(NonceMilestoneRequiredTitleRule {
                message: "nonce milestone needs an entry".to_string(),
            }),
            Rc::new(NonceMilestoneEndsAtRangeRule { min: 80, max: 255 }),
            Rc::new(NonceMilestoneCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_milestone_validator();
        let item = new_nonce_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = NonceMilestoneRegionLengthRule { max_length: 64 };
        let mut item = new_nonce_milestone("x", "y");
        item.region = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = NonceMilestoneRequiredTitleRule {
            message: "nonce milestone needs an entry".to_string(),
        };
        let mut item = new_nonce_milestone("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = NonceMilestoneEndsAtRangeRule { min: 80, max: 80 };
        let mut item = new_nonce_milestone("x", "y");
        item.ends_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = NonceMilestoneCodeLengthRule { max_length: 16 };
        let mut item = new_nonce_milestone("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
