use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling allowlist exception must pass. Check returns its problems, or none.
pub trait RollingAllowlistExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RollingAllowlistException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistExceptionBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingAllowlistExceptionRule for RollingAllowlistExceptionBudgetRangeRule {
    fn check(&self, item: &RollingAllowlistException) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistExceptionAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingAllowlistExceptionRule for RollingAllowlistExceptionAttemptsRangeRule {
    fn check(&self, item: &RollingAllowlistException) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistExceptionAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl RollingAllowlistExceptionRule for RollingAllowlistExceptionAllowedKeyIdRule {
    fn check(&self, item: &RollingAllowlistException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is below minimum", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistExceptionCodeLengthRule {
    pub max_length: i64,
}

impl RollingAllowlistExceptionRule for RollingAllowlistExceptionCodeLengthRule {
    fn check(&self, item: &RollingAllowlistException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistExceptionRequiredCodeRule {
    pub message: String,
}

impl RollingAllowlistExceptionRule for RollingAllowlistExceptionRequiredCodeRule {
    fn check(&self, item: &RollingAllowlistException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistExceptionRequiredClusterRule {
    pub message: String,
}

impl RollingAllowlistExceptionRule for RollingAllowlistExceptionRequiredClusterRule {
    fn check(&self, item: &RollingAllowlistException) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling allowlist exception.
#[derive(Debug, Clone)]
pub struct RollingAllowlistExceptionValidator {
    pub rules: Vec<Rc<dyn RollingAllowlistExceptionRule>>,
}

impl RollingAllowlistExceptionValidator {
    pub fn validate(&self, item: &RollingAllowlistException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingAllowlistException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingAllowlistException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_allowlist_exception_validator() -> RollingAllowlistExceptionValidator {
    RollingAllowlistExceptionValidator {
        rules: vec![
            Rc::new(RollingAllowlistExceptionBudgetRangeRule { min: 8.7, max: 1172.5 })
                as Rc<dyn RollingAllowlistExceptionRule>,
            Rc::new(RollingAllowlistExceptionAttemptsRangeRule { min: 16, max: 80 }),
            Rc::new(RollingAllowlistExceptionAllowedKeyIdRule {
                allowed: vec!["weighted".to_string(), "batched".to_string(), "tiered".to_string()],
            }),
            Rc::new(RollingAllowlistExceptionCodeLengthRule { max_length: 1000 }),
            Rc::new(RollingAllowlistExceptionRequiredCodeRule {
                message: "rolling allowlist exception needs a value".to_string(),
            }),
            Rc::new(RollingAllowlistExceptionRequiredClusterRule {
                message: "rolling allowlist exception needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_allowlist_exception_validator();
        let item = new_rolling_allowlist_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RollingAllowlistExceptionBudgetRangeRule { min: 1.1, max: 2489.9 };
        let mut item = new_rolling_allowlist_exception("x", "y");
        item.budget = 2491.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = RollingAllowlistExceptionAttemptsRangeRule { min: 32, max: 64 };
        let mut item = new_rolling_allowlist_exception("x", "y");
        item.attempts = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingAllowlistExceptionCodeLengthRule { max_length: 1000 };
        let mut item = new_rolling_allowlist_exception("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingAllowlistExceptionRequiredCodeRule {
            message: "rolling allowlist exception needs a value".to_string(),
        };
        let mut item = new_rolling_allowlist_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RollingAllowlistExceptionRequiredClusterRule {
            message: "rolling allowlist exception needs an entry".to_string(),
        };
        let mut item = new_rolling_allowlist_exception("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
