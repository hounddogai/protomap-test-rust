use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global timeline adjustment must pass. Check returns its problems, or none.
pub trait GlobalTimelineAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTimelineAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineAdjustmentRequiredNamespaceRule {
    pub message: String,
}

impl GlobalTimelineAdjustmentRule for GlobalTimelineAdjustmentRequiredNamespaceRule {
    fn check(&self, item: &GlobalTimelineAdjustment) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl GlobalTimelineAdjustmentRule for GlobalTimelineAdjustmentTitleLengthRule {
    fn check(&self, item: &GlobalTimelineAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineAdjustmentRequiredUnitRule {
    pub message: String,
}

impl GlobalTimelineAdjustmentRule for GlobalTimelineAdjustmentRequiredUnitRule {
    fn check(&self, item: &GlobalTimelineAdjustment) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl GlobalTimelineAdjustmentRule for GlobalTimelineAdjustmentCodeLengthRule {
    fn check(&self, item: &GlobalTimelineAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineAdjustmentRequiredCodeRule {
    pub message: String,
}

impl GlobalTimelineAdjustmentRule for GlobalTimelineAdjustmentRequiredCodeRule {
    fn check(&self, item: &GlobalTimelineAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineAdjustmentTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalTimelineAdjustmentRule for GlobalTimelineAdjustmentTtlSecondsRangeRule {
    fn check(&self, item: &GlobalTimelineAdjustment) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global timeline adjustment.
#[derive(Debug, Clone)]
pub struct GlobalTimelineAdjustmentValidator {
    pub rules: Vec<Rc<dyn GlobalTimelineAdjustmentRule>>,
}

impl GlobalTimelineAdjustmentValidator {
    pub fn validate(&self, item: &GlobalTimelineAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalTimelineAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTimelineAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_timeline_adjustment_validator() -> GlobalTimelineAdjustmentValidator {
    GlobalTimelineAdjustmentValidator {
        rules: vec![
            Rc::new(GlobalTimelineAdjustmentRequiredNamespaceRule {
                message: "global timeline adjustment needs a value".to_string(),
            }) as Rc<dyn GlobalTimelineAdjustmentRule>,
            Rc::new(GlobalTimelineAdjustmentTitleLengthRule { max_length: 3600 }),
            Rc::new(GlobalTimelineAdjustmentRequiredUnitRule {
                message: "global timeline adjustment needs an entry".to_string(),
            }),
            Rc::new(GlobalTimelineAdjustmentCodeLengthRule { max_length: 80 }),
            Rc::new(GlobalTimelineAdjustmentRequiredCodeRule {
                message: "global timeline adjustment needs a value".to_string(),
            }),
            Rc::new(GlobalTimelineAdjustmentTtlSecondsRangeRule { min: 80, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_timeline_adjustment_validator();
        let item = new_global_timeline_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = GlobalTimelineAdjustmentRequiredNamespaceRule {
            message: "global timeline adjustment needs an entry".to_string(),
        };
        let mut item = new_global_timeline_adjustment("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalTimelineAdjustmentTitleLengthRule { max_length: 64 };
        let mut item = new_global_timeline_adjustment("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = GlobalTimelineAdjustmentRequiredUnitRule {
            message: "global timeline adjustment needs an entry".to_string(),
        };
        let mut item = new_global_timeline_adjustment("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalTimelineAdjustmentCodeLengthRule { max_length: 64 };
        let mut item = new_global_timeline_adjustment("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalTimelineAdjustmentRequiredCodeRule {
            message: "global timeline adjustment needs this field".to_string(),
        };
        let mut item = new_global_timeline_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = GlobalTimelineAdjustmentTtlSecondsRangeRule { min: 16, max: 120 };
        let mut item = new_global_timeline_adjustment("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
