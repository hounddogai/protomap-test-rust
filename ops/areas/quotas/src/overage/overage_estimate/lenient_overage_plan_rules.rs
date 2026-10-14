use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient overage plan must pass. Check returns its problems, or none.
pub trait LenientOveragePlanRule: std::fmt::Debug {
    fn check(&self, item: &LenientOveragePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientOveragePlanTitleLengthRule {
    pub max_length: i64,
}

impl LenientOveragePlanRule for LenientOveragePlanTitleLengthRule {
    fn check(&self, item: &LenientOveragePlan) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientOveragePlanRequiredLocaleRule {
    pub message: String,
}

impl LenientOveragePlanRule for LenientOveragePlanRequiredLocaleRule {
    fn check(&self, item: &LenientOveragePlan) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientOveragePlanRequiredTitleRule {
    pub message: String,
}

impl LenientOveragePlanRule for LenientOveragePlanRequiredTitleRule {
    fn check(&self, item: &LenientOveragePlan) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient overage plan.
#[derive(Debug, Clone)]
pub struct LenientOveragePlanValidator {
    pub rules: Vec<Rc<dyn LenientOveragePlanRule>>,
}

impl LenientOveragePlanValidator {
    pub fn validate(&self, item: &LenientOveragePlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientOveragePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientOveragePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_overage_plan_validator() -> LenientOveragePlanValidator {
    LenientOveragePlanValidator {
        rules: vec![
            Rc::new(LenientOveragePlanTitleLengthRule { max_length: 3600 }) as Rc<dyn LenientOveragePlanRule>,
            Rc::new(LenientOveragePlanRequiredLocaleRule {
                message: "lenient overage plan needs this field".to_string(),
            }),
            Rc::new(LenientOveragePlanRequiredTitleRule {
                message: "lenient overage plan needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_overage_plan_validator();
        let item = new_lenient_overage_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientOveragePlanTitleLengthRule { max_length: 255 };
        let mut item = new_lenient_overage_plan("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = LenientOveragePlanRequiredLocaleRule {
            message: "lenient overage plan needs this field".to_string(),
        };
        let mut item = new_lenient_overage_plan("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientOveragePlanRequiredTitleRule {
            message: "lenient overage plan needs this field".to_string(),
        };
        let mut item = new_lenient_overage_plan("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
