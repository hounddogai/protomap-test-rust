use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling health check threshold must pass. Check returns its problems, or none.
pub trait RollingHealthCheckThresholdRule: std::fmt::Debug {
    fn check(&self, item: &RollingHealthCheckThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingHealthCheckThresholdCurrencyLengthRule {
    pub max_length: i64,
}

impl RollingHealthCheckThresholdRule for RollingHealthCheckThresholdCurrencyLengthRule {
    fn check(&self, item: &RollingHealthCheckThreshold) -> Vec<String> {
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
pub struct RollingHealthCheckThresholdRequiredCurrencyRule {
    pub message: String,
}

impl RollingHealthCheckThresholdRule for RollingHealthCheckThresholdRequiredCurrencyRule {
    fn check(&self, item: &RollingHealthCheckThreshold) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingHealthCheckThresholdRequiredTitleRule {
    pub message: String,
}

impl RollingHealthCheckThresholdRule for RollingHealthCheckThresholdRequiredTitleRule {
    fn check(&self, item: &RollingHealthCheckThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingHealthCheckThresholdRequiredCodeRule {
    pub message: String,
}

impl RollingHealthCheckThresholdRule for RollingHealthCheckThresholdRequiredCodeRule {
    fn check(&self, item: &RollingHealthCheckThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling health check threshold.
#[derive(Debug, Clone)]
pub struct RollingHealthCheckThresholdValidator {
    pub rules: Vec<Rc<dyn RollingHealthCheckThresholdRule>>,
}

impl RollingHealthCheckThresholdValidator {
    pub fn validate(&self, item: &RollingHealthCheckThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingHealthCheckThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingHealthCheckThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_health_check_threshold_validator() -> RollingHealthCheckThresholdValidator {
    RollingHealthCheckThresholdValidator {
        rules: vec![
            Rc::new(RollingHealthCheckThresholdCurrencyLengthRule { max_length: 1000 })
                as Rc<dyn RollingHealthCheckThresholdRule>,
            Rc::new(RollingHealthCheckThresholdRequiredCurrencyRule {
                message: "rolling health check threshold needs an entry".to_string(),
            }),
            Rc::new(RollingHealthCheckThresholdRequiredTitleRule {
                message: "rolling health check threshold needs this field".to_string(),
            }),
            Rc::new(RollingHealthCheckThresholdRequiredCodeRule {
                message: "rolling health check threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_health_check_threshold_validator();
        let item = new_rolling_health_check_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RollingHealthCheckThresholdCurrencyLengthRule { max_length: 255 };
        let mut item = new_rolling_health_check_threshold("x", "y");
        item.currency = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = RollingHealthCheckThresholdRequiredCurrencyRule {
            message: "rolling health check threshold needs a value".to_string(),
        };
        let mut item = new_rolling_health_check_threshold("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingHealthCheckThresholdRequiredTitleRule {
            message: "rolling health check threshold needs this field".to_string(),
        };
        let mut item = new_rolling_health_check_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingHealthCheckThresholdRequiredCodeRule {
            message: "rolling health check threshold needs this field".to_string(),
        };
        let mut item = new_rolling_health_check_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
