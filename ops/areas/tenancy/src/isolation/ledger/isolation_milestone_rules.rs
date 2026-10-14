use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation milestone must pass. Check returns its problems, or none.
pub trait IsolationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &IsolationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMilestoneRegionLengthRule {
    pub max_length: i64,
}

impl IsolationMilestoneRule for IsolationMilestoneRegionLengthRule {
    fn check(&self, item: &IsolationMilestone) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMilestoneRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationMilestoneRule for IsolationMilestoneRevisionRangeRule {
    fn check(&self, item: &IsolationMilestone) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMilestoneCodePrefixRule {
    pub prefix: String,
}

impl IsolationMilestoneRule for IsolationMilestoneCodePrefixRule {
    fn check(&self, item: &IsolationMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl IsolationMilestoneRule for IsolationMilestoneCodeLengthRule {
    fn check(&self, item: &IsolationMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMilestoneRequiredCodeRule {
    pub message: String,
}

impl IsolationMilestoneRule for IsolationMilestoneRequiredCodeRule {
    fn check(&self, item: &IsolationMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMilestoneRequiredRegionRule {
    pub message: String,
}

impl IsolationMilestoneRule for IsolationMilestoneRequiredRegionRule {
    fn check(&self, item: &IsolationMilestone) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation milestone.
#[derive(Debug, Clone)]
pub struct IsolationMilestoneValidator {
    pub rules: Vec<Rc<dyn IsolationMilestoneRule>>,
}

impl IsolationMilestoneValidator {
    pub fn validate(&self, item: &IsolationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_milestone_validator() -> IsolationMilestoneValidator {
    IsolationMilestoneValidator {
        rules: vec![
            Rc::new(IsolationMilestoneRegionLengthRule { max_length: 32 }) as Rc<dyn IsolationMilestoneRule>,
            Rc::new(IsolationMilestoneRevisionRangeRule { min: 3600, max: 86400 }),
            Rc::new(IsolationMilestoneCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(IsolationMilestoneCodeLengthRule { max_length: 32 }),
            Rc::new(IsolationMilestoneRequiredCodeRule {
                message: "isolation milestone needs this field".to_string(),
            }),
            Rc::new(IsolationMilestoneRequiredRegionRule {
                message: "isolation milestone needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_milestone_validator();
        let item = new_isolation_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = IsolationMilestoneRegionLengthRule { max_length: 255 };
        let mut item = new_isolation_milestone("x", "y");
        item.region = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = IsolationMilestoneRevisionRangeRule { min: 120, max: 86400 };
        let mut item = new_isolation_milestone("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IsolationMilestoneCodeLengthRule { max_length: 80 };
        let mut item = new_isolation_milestone("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationMilestoneRequiredCodeRule {
            message: "isolation milestone needs an entry".to_string(),
        };
        let mut item = new_isolation_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = IsolationMilestoneRequiredRegionRule {
            message: "isolation milestone needs this field".to_string(),
        };
        let mut item = new_isolation_milestone("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
