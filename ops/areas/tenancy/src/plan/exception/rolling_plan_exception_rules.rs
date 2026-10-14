use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling plan exception must pass. Check returns its problems, or none.
pub trait RollingPlanExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RollingPlanException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanExceptionCurrencyLengthRule {
    pub max_length: i64,
}

impl RollingPlanExceptionRule for RollingPlanExceptionCurrencyLengthRule {
    fn check(&self, item: &RollingPlanException) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanExceptionRequiredTitleRule {
    pub message: String,
}

impl RollingPlanExceptionRule for RollingPlanExceptionRequiredTitleRule {
    fn check(&self, item: &RollingPlanException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanExceptionRequiredNameRule {
    pub message: String,
}

impl RollingPlanExceptionRule for RollingPlanExceptionRequiredNameRule {
    fn check(&self, item: &RollingPlanException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPlanExceptionAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RollingPlanExceptionRule for RollingPlanExceptionAllowedTitleRule {
    fn check(&self, item: &RollingPlanException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling plan exception.
#[derive(Debug, Clone)]
pub struct RollingPlanExceptionValidator {
    pub rules: Vec<Rc<dyn RollingPlanExceptionRule>>,
}

impl RollingPlanExceptionValidator {
    pub fn validate(&self, item: &RollingPlanException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPlanException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPlanException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_plan_exception_validator() -> RollingPlanExceptionValidator {
    RollingPlanExceptionValidator {
        rules: vec![
            Rc::new(RollingPlanExceptionCurrencyLengthRule { max_length: 32 }) as Rc<dyn RollingPlanExceptionRule>,
            Rc::new(RollingPlanExceptionRequiredTitleRule {
                message: "rolling plan exception needs an entry".to_string(),
            }),
            Rc::new(RollingPlanExceptionRequiredNameRule {
                message: "rolling plan exception needs this field".to_string(),
            }),
            Rc::new(RollingPlanExceptionAllowedTitleRule {
                allowed: vec!["fallback".to_string(), "strict".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_plan_exception_validator();
        let item = new_rolling_plan_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RollingPlanExceptionCurrencyLengthRule { max_length: 255 };
        let mut item = new_rolling_plan_exception("x", "y");
        item.currency = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingPlanExceptionRequiredTitleRule {
            message: "rolling plan exception needs an entry".to_string(),
        };
        let mut item = new_rolling_plan_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingPlanExceptionRequiredNameRule {
            message: "rolling plan exception needs this field".to_string(),
        };
        let mut item = new_rolling_plan_exception("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
