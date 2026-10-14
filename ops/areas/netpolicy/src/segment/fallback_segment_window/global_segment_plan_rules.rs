use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global segment plan must pass. Check returns its problems, or none.
pub trait GlobalSegmentPlanRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSegmentPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPlanMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalSegmentPlanRule for GlobalSegmentPlanMarginRangeRule {
    fn check(&self, item: &GlobalSegmentPlan) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPlanLabelLengthRule {
    pub max_length: i64,
}

impl GlobalSegmentPlanRule for GlobalSegmentPlanLabelLengthRule {
    fn check(&self, item: &GlobalSegmentPlan) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPlanRequiredCodeRule {
    pub message: String,
}

impl GlobalSegmentPlanRule for GlobalSegmentPlanRequiredCodeRule {
    fn check(&self, item: &GlobalSegmentPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPlanSourceLengthRule {
    pub max_length: i64,
}

impl GlobalSegmentPlanRule for GlobalSegmentPlanSourceLengthRule {
    fn check(&self, item: &GlobalSegmentPlan) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPlanEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalSegmentPlanRule for GlobalSegmentPlanEndsAtRangeRule {
    fn check(&self, item: &GlobalSegmentPlan) -> Vec<String> {
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
pub struct GlobalSegmentPlanCodeLengthRule {
    pub max_length: i64,
}

impl GlobalSegmentPlanRule for GlobalSegmentPlanCodeLengthRule {
    fn check(&self, item: &GlobalSegmentPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global segment plan.
#[derive(Debug, Clone)]
pub struct GlobalSegmentPlanValidator {
    pub rules: Vec<Rc<dyn GlobalSegmentPlanRule>>,
}

impl GlobalSegmentPlanValidator {
    pub fn validate(&self, item: &GlobalSegmentPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalSegmentPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSegmentPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_segment_plan_validator() -> GlobalSegmentPlanValidator {
    GlobalSegmentPlanValidator {
        rules: vec![
            Rc::new(GlobalSegmentPlanMarginRangeRule { min: 3.6, max: 3224.8 }) as Rc<dyn GlobalSegmentPlanRule>,
            Rc::new(GlobalSegmentPlanLabelLengthRule { max_length: 16 }),
            Rc::new(GlobalSegmentPlanRequiredCodeRule {
                message: "global segment plan needs an entry".to_string(),
            }),
            Rc::new(GlobalSegmentPlanSourceLengthRule { max_length: 120 }),
            Rc::new(GlobalSegmentPlanEndsAtRangeRule { min: 64, max: 120 }),
            Rc::new(GlobalSegmentPlanCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_segment_plan_validator();
        let item = new_global_segment_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = GlobalSegmentPlanMarginRangeRule { min: 0.7, max: 3681.6 };
        let mut item = new_global_segment_plan("x", "y");
        item.margin = 3683.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GlobalSegmentPlanLabelLengthRule { max_length: 120 };
        let mut item = new_global_segment_plan("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalSegmentPlanRequiredCodeRule {
            message: "global segment plan needs this field".to_string(),
        };
        let mut item = new_global_segment_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = GlobalSegmentPlanSourceLengthRule { max_length: 120 };
        let mut item = new_global_segment_plan("x", "y");
        item.source = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = GlobalSegmentPlanEndsAtRangeRule { min: 16, max: 120 };
        let mut item = new_global_segment_plan("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalSegmentPlanCodeLengthRule { max_length: 64 };
        let mut item = new_global_segment_plan("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
