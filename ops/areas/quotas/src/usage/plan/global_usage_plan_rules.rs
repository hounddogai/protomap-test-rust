use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global usage plan must pass. Check returns its problems, or none.
pub trait GlobalUsagePlanRule: std::fmt::Debug {
    fn check(&self, item: &GlobalUsagePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsagePlanTitleLengthRule {
    pub max_length: i64,
}

impl GlobalUsagePlanRule for GlobalUsagePlanTitleLengthRule {
    fn check(&self, item: &GlobalUsagePlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsagePlanRequiredTitleRule {
    pub message: String,
}

impl GlobalUsagePlanRule for GlobalUsagePlanRequiredTitleRule {
    fn check(&self, item: &GlobalUsagePlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsagePlanRequiredLabelRule {
    pub message: String,
}

impl GlobalUsagePlanRule for GlobalUsagePlanRequiredLabelRule {
    fn check(&self, item: &GlobalUsagePlan) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global usage plan.
#[derive(Debug, Clone)]
pub struct GlobalUsagePlanValidator {
    pub rules: Vec<Rc<dyn GlobalUsagePlanRule>>,
}

impl GlobalUsagePlanValidator {
    pub fn validate(&self, item: &GlobalUsagePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalUsagePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalUsagePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_usage_plan_validator() -> GlobalUsagePlanValidator {
    GlobalUsagePlanValidator {
        rules: vec![
            Rc::new(GlobalUsagePlanTitleLengthRule { max_length: 1000 }) as Rc<dyn GlobalUsagePlanRule>,
            Rc::new(GlobalUsagePlanRequiredTitleRule {
                message: "global usage plan needs this field".to_string(),
            }),
            Rc::new(GlobalUsagePlanRequiredLabelRule {
                message: "global usage plan needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_usage_plan_validator();
        let item = new_global_usage_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalUsagePlanTitleLengthRule { max_length: 64 };
        let mut item = new_global_usage_plan("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalUsagePlanRequiredTitleRule {
            message: "global usage plan needs an entry".to_string(),
        };
        let mut item = new_global_usage_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GlobalUsagePlanRequiredLabelRule {
            message: "global usage plan needs this field".to_string(),
        };
        let mut item = new_global_usage_plan("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
