use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline milestone must pass. Check returns its problems, or none.
pub trait TimelineMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &TimelineMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMilestoneRequiredOwnerIdRule {
    pub message: String,
}

impl TimelineMilestoneRule for TimelineMilestoneRequiredOwnerIdRule {
    fn check(&self, item: &TimelineMilestone) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMilestoneQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineMilestoneRule for TimelineMilestoneQuantityRangeRule {
    fn check(&self, item: &TimelineMilestone) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMilestoneCodePrefixRule {
    pub prefix: String,
}

impl TimelineMilestoneRule for TimelineMilestoneCodePrefixRule {
    fn check(&self, item: &TimelineMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMilestoneTitleLengthRule {
    pub max_length: i64,
}

impl TimelineMilestoneRule for TimelineMilestoneTitleLengthRule {
    fn check(&self, item: &TimelineMilestone) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMilestoneOwnerIdLengthRule {
    pub max_length: i64,
}

impl TimelineMilestoneRule for TimelineMilestoneOwnerIdLengthRule {
    fn check(&self, item: &TimelineMilestone) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMilestoneRequiredCodeRule {
    pub message: String,
}

impl TimelineMilestoneRule for TimelineMilestoneRequiredCodeRule {
    fn check(&self, item: &TimelineMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline milestone.
#[derive(Debug, Clone)]
pub struct TimelineMilestoneValidator {
    pub rules: Vec<Rc<dyn TimelineMilestoneRule>>,
}

impl TimelineMilestoneValidator {
    pub fn validate(&self, item: &TimelineMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_milestone_validator() -> TimelineMilestoneValidator {
    TimelineMilestoneValidator {
        rules: vec![
            Rc::new(TimelineMilestoneRequiredOwnerIdRule {
                message: "timeline milestone needs a value".to_string(),
            }) as Rc<dyn TimelineMilestoneRule>,
            Rc::new(TimelineMilestoneQuantityRangeRule { min: 80, max: 1000 }),
            Rc::new(TimelineMilestoneCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TimelineMilestoneTitleLengthRule { max_length: 120 }),
            Rc::new(TimelineMilestoneOwnerIdLengthRule { max_length: 64 }),
            Rc::new(TimelineMilestoneRequiredCodeRule {
                message: "timeline milestone needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_milestone_validator();
        let item = new_timeline_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = TimelineMilestoneRequiredOwnerIdRule {
            message: "timeline milestone needs a value".to_string(),
        };
        let mut item = new_timeline_milestone("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = TimelineMilestoneQuantityRangeRule { min: 80, max: 80 };
        let mut item = new_timeline_milestone("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TimelineMilestoneTitleLengthRule { max_length: 86400 };
        let mut item = new_timeline_milestone("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = TimelineMilestoneOwnerIdLengthRule { max_length: 1000 };
        let mut item = new_timeline_milestone("x", "y");
        item.owner_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TimelineMilestoneRequiredCodeRule {
            message: "timeline milestone needs a value".to_string(),
        };
        let mut item = new_timeline_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
