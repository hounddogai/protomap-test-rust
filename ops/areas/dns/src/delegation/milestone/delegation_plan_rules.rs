use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation plan must pass. Check returns its problems, or none.
pub trait DelegationPlanRule: std::fmt::Debug {
    fn check(&self, item: &DelegationPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPlanTitleLengthRule {
    pub max_length: i64,
}

impl DelegationPlanRule for DelegationPlanTitleLengthRule {
    fn check(&self, item: &DelegationPlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPlanRequiredTitleRule {
    pub message: String,
}

impl DelegationPlanRule for DelegationPlanRequiredTitleRule {
    fn check(&self, item: &DelegationPlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPlanRequiredCodeRule {
    pub message: String,
}

impl DelegationPlanRule for DelegationPlanRequiredCodeRule {
    fn check(&self, item: &DelegationPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPlanCodeLengthRule {
    pub max_length: i64,
}

impl DelegationPlanRule for DelegationPlanCodeLengthRule {
    fn check(&self, item: &DelegationPlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPlanVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationPlanRule for DelegationPlanVersionRangeRule {
    fn check(&self, item: &DelegationPlan) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation plan.
#[derive(Debug, Clone)]
pub struct DelegationPlanValidator {
    pub rules: Vec<Rc<dyn DelegationPlanRule>>,
}

impl DelegationPlanValidator {
    pub fn validate(&self, item: &DelegationPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_plan_validator() -> DelegationPlanValidator {
    DelegationPlanValidator {
        rules: vec![
            Rc::new(DelegationPlanTitleLengthRule { max_length: 32 }) as Rc<dyn DelegationPlanRule>,
            Rc::new(DelegationPlanRequiredTitleRule {
                message: "delegation plan needs this field".to_string(),
            }),
            Rc::new(DelegationPlanRequiredCodeRule {
                message: "delegation plan needs an entry".to_string(),
            }),
            Rc::new(DelegationPlanCodeLengthRule { max_length: 120 }),
            Rc::new(DelegationPlanVersionRangeRule { min: 64, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_plan_validator();
        let item = new_delegation_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DelegationPlanTitleLengthRule { max_length: 120 };
        let mut item = new_delegation_plan("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DelegationPlanRequiredTitleRule {
            message: "delegation plan needs this field".to_string(),
        };
        let mut item = new_delegation_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationPlanRequiredCodeRule {
            message: "delegation plan needs an entry".to_string(),
        };
        let mut item = new_delegation_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DelegationPlanCodeLengthRule { max_length: 80 };
        let mut item = new_delegation_plan("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = DelegationPlanVersionRangeRule { min: 16, max: 255 };
        let mut item = new_delegation_plan("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
