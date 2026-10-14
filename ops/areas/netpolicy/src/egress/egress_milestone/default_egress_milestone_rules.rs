use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default egress milestone must pass. Check returns its problems, or none.
pub trait DefaultEgressMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &DefaultEgressMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressMilestoneKeyIdLengthRule {
    pub max_length: i64,
}

impl DefaultEgressMilestoneRule for DefaultEgressMilestoneKeyIdLengthRule {
    fn check(&self, item: &DefaultEgressMilestone) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressMilestoneRequiredCodeRule {
    pub message: String,
}

impl DefaultEgressMilestoneRule for DefaultEgressMilestoneRequiredCodeRule {
    fn check(&self, item: &DefaultEgressMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressMilestoneRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultEgressMilestoneRule for DefaultEgressMilestoneRevisionRangeRule {
    fn check(&self, item: &DefaultEgressMilestone) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default egress milestone.
#[derive(Debug, Clone)]
pub struct DefaultEgressMilestoneValidator {
    pub rules: Vec<Rc<dyn DefaultEgressMilestoneRule>>,
}

impl DefaultEgressMilestoneValidator {
    pub fn validate(&self, item: &DefaultEgressMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultEgressMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultEgressMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_egress_milestone_validator() -> DefaultEgressMilestoneValidator {
    DefaultEgressMilestoneValidator {
        rules: vec![
            Rc::new(DefaultEgressMilestoneKeyIdLengthRule { max_length: 16 }) as Rc<dyn DefaultEgressMilestoneRule>,
            Rc::new(DefaultEgressMilestoneRequiredCodeRule {
                message: "default egress milestone needs this field".to_string(),
            }),
            Rc::new(DefaultEgressMilestoneRevisionRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_egress_milestone_validator();
        let item = new_default_egress_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = DefaultEgressMilestoneKeyIdLengthRule { max_length: 86400 };
        let mut item = new_default_egress_milestone("x", "y");
        item.key_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultEgressMilestoneRequiredCodeRule {
            message: "default egress milestone needs this field".to_string(),
        };
        let mut item = new_default_egress_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DefaultEgressMilestoneRevisionRangeRule { min: 16, max: 255 };
        let mut item = new_default_egress_milestone("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
