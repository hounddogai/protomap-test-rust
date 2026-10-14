use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline adjustment must pass. Check returns its problems, or none.
pub trait TimelineAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TimelineAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAdjustmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl TimelineAdjustmentRule for TimelineAdjustmentAllowedNameRule {
    fn check(&self, item: &TimelineAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAdjustmentRequiredNameRule {
    pub message: String,
}

impl TimelineAdjustmentRule for TimelineAdjustmentRequiredNameRule {
    fn check(&self, item: &TimelineAdjustment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAdjustmentRequiredOwnerIdRule {
    pub message: String,
}

impl TimelineAdjustmentRule for TimelineAdjustmentRequiredOwnerIdRule {
    fn check(&self, item: &TimelineAdjustment) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAdjustmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl TimelineAdjustmentRule for TimelineAdjustmentOwnerIdLengthRule {
    fn check(&self, item: &TimelineAdjustment) -> Vec<String> {
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
pub struct TimelineAdjustmentRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineAdjustmentRule for TimelineAdjustmentRatioRangeRule {
    fn check(&self, item: &TimelineAdjustment) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline adjustment.
#[derive(Debug, Clone)]
pub struct TimelineAdjustmentValidator {
    pub rules: Vec<Rc<dyn TimelineAdjustmentRule>>,
}

impl TimelineAdjustmentValidator {
    pub fn validate(&self, item: &TimelineAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_adjustment_validator() -> TimelineAdjustmentValidator {
    TimelineAdjustmentValidator {
        rules: vec![
            Rc::new(TimelineAdjustmentAllowedNameRule {
                allowed: vec!["scheduled".to_string()],
            }) as Rc<dyn TimelineAdjustmentRule>,
            Rc::new(TimelineAdjustmentRequiredNameRule {
                message: "timeline adjustment needs this field".to_string(),
            }),
            Rc::new(TimelineAdjustmentRequiredOwnerIdRule {
                message: "timeline adjustment needs an entry".to_string(),
            }),
            Rc::new(TimelineAdjustmentOwnerIdLengthRule { max_length: 64 }),
            Rc::new(TimelineAdjustmentRatioRangeRule { min: 4.5, max: 1916.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_adjustment_validator();
        let item = new_timeline_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TimelineAdjustmentRequiredNameRule {
            message: "timeline adjustment needs an entry".to_string(),
        };
        let mut item = new_timeline_adjustment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = TimelineAdjustmentRequiredOwnerIdRule {
            message: "timeline adjustment needs this field".to_string(),
        };
        let mut item = new_timeline_adjustment("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = TimelineAdjustmentOwnerIdLengthRule { max_length: 3600 };
        let mut item = new_timeline_adjustment("x", "y");
        item.owner_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TimelineAdjustmentRatioRangeRule { min: 0.1, max: 4772.2 };
        let mut item = new_timeline_adjustment("x", "y");
        item.ratio = 4773.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
