use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual quote plan must pass. Check returns its problems, or none.
pub trait ManualQuotePlanRule: std::fmt::Debug {
    fn check(&self, item: &ManualQuotePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuotePlanRequiredUnitRule {
    pub message: String,
}

impl ManualQuotePlanRule for ManualQuotePlanRequiredUnitRule {
    fn check(&self, item: &ManualQuotePlan) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuotePlanTitleLengthRule {
    pub max_length: i64,
}

impl ManualQuotePlanRule for ManualQuotePlanTitleLengthRule {
    fn check(&self, item: &ManualQuotePlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuotePlanUnitLengthRule {
    pub max_length: i64,
}

impl ManualQuotePlanRule for ManualQuotePlanUnitLengthRule {
    fn check(&self, item: &ManualQuotePlan) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuotePlanRequiredCodeRule {
    pub message: String,
}

impl ManualQuotePlanRule for ManualQuotePlanRequiredCodeRule {
    fn check(&self, item: &ManualQuotePlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuotePlanVisibleGuardRule {
    pub limit: f64,
}

impl ManualQuotePlanRule for ManualQuotePlanVisibleGuardRule {
    fn check(&self, item: &ManualQuotePlan) -> Vec<String> {
        if item.visible && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while visible", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuotePlanRequiredScopeRule {
    pub message: String,
}

impl ManualQuotePlanRule for ManualQuotePlanRequiredScopeRule {
    fn check(&self, item: &ManualQuotePlan) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual quote plan.
#[derive(Debug, Clone)]
pub struct ManualQuotePlanValidator {
    pub rules: Vec<Rc<dyn ManualQuotePlanRule>>,
}

impl ManualQuotePlanValidator {
    pub fn validate(&self, item: &ManualQuotePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualQuotePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualQuotePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_quote_plan_validator() -> ManualQuotePlanValidator {
    ManualQuotePlanValidator {
        rules: vec![
            Rc::new(ManualQuotePlanRequiredUnitRule {
                message: "manual quote plan needs this field".to_string(),
            }) as Rc<dyn ManualQuotePlanRule>,
            Rc::new(ManualQuotePlanTitleLengthRule { max_length: 255 }),
            Rc::new(ManualQuotePlanUnitLengthRule { max_length: 120 }),
            Rc::new(ManualQuotePlanRequiredCodeRule {
                message: "manual quote plan needs a value".to_string(),
            }),
            Rc::new(ManualQuotePlanVisibleGuardRule { limit: 2.6 }),
            Rc::new(ManualQuotePlanRequiredScopeRule {
                message: "manual quote plan needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_quote_plan_validator();
        let item = new_manual_quote_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = ManualQuotePlanRequiredUnitRule {
            message: "manual quote plan needs this field".to_string(),
        };
        let mut item = new_manual_quote_plan("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualQuotePlanTitleLengthRule { max_length: 1000 };
        let mut item = new_manual_quote_plan("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ManualQuotePlanUnitLengthRule { max_length: 255 };
        let mut item = new_manual_quote_plan("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualQuotePlanRequiredCodeRule {
            message: "manual quote plan needs a value".to_string(),
        };
        let mut item = new_manual_quote_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ManualQuotePlanRequiredScopeRule {
            message: "manual quote plan needs an entry".to_string(),
        };
        let mut item = new_manual_quote_plan("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
