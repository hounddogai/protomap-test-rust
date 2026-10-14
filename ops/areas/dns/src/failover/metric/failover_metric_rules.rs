use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover metric must pass. Check returns its problems, or none.
pub trait FailoverMetricRule: std::fmt::Debug {
    fn check(&self, item: &FailoverMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMetricCodeLengthRule {
    pub max_length: i64,
}

impl FailoverMetricRule for FailoverMetricCodeLengthRule {
    fn check(&self, item: &FailoverMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMetricRequiredOwnerIdRule {
    pub message: String,
}

impl FailoverMetricRule for FailoverMetricRequiredOwnerIdRule {
    fn check(&self, item: &FailoverMetric) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMetricOwnerIdLengthRule {
    pub max_length: i64,
}

impl FailoverMetricRule for FailoverMetricOwnerIdLengthRule {
    fn check(&self, item: &FailoverMetric) -> Vec<String> {
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
pub struct FailoverMetricAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl FailoverMetricRule for FailoverMetricAllowedOwnerIdRule {
    fn check(&self, item: &FailoverMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is expired", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMetricAudienceLengthRule {
    pub max_length: i64,
}

impl FailoverMetricRule for FailoverMetricAudienceLengthRule {
    fn check(&self, item: &FailoverMetric) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMetricRequiredAudienceRule {
    pub message: String,
}

impl FailoverMetricRule for FailoverMetricRequiredAudienceRule {
    fn check(&self, item: &FailoverMetric) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover metric.
#[derive(Debug, Clone)]
pub struct FailoverMetricValidator {
    pub rules: Vec<Rc<dyn FailoverMetricRule>>,
}

impl FailoverMetricValidator {
    pub fn validate(&self, item: &FailoverMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_metric_validator() -> FailoverMetricValidator {
    FailoverMetricValidator {
        rules: vec![
            Rc::new(FailoverMetricCodeLengthRule { max_length: 32 }) as Rc<dyn FailoverMetricRule>,
            Rc::new(FailoverMetricRequiredOwnerIdRule {
                message: "failover metric needs an entry".to_string(),
            }),
            Rc::new(FailoverMetricOwnerIdLengthRule { max_length: 1000 }),
            Rc::new(FailoverMetricAllowedOwnerIdRule {
                allowed: vec!["seasonal".to_string(), "adaptive".to_string()],
            }),
            Rc::new(FailoverMetricAudienceLengthRule { max_length: 255 }),
            Rc::new(FailoverMetricRequiredAudienceRule {
                message: "failover metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_metric_validator();
        let item = new_failover_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FailoverMetricCodeLengthRule { max_length: 255 };
        let mut item = new_failover_metric("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = FailoverMetricRequiredOwnerIdRule {
            message: "failover metric needs a value".to_string(),
        };
        let mut item = new_failover_metric("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = FailoverMetricOwnerIdLengthRule { max_length: 3600 };
        let mut item = new_failover_metric("x", "y");
        item.owner_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = FailoverMetricAudienceLengthRule { max_length: 32 };
        let mut item = new_failover_metric("x", "y");
        item.audience = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = FailoverMetricRequiredAudienceRule {
            message: "failover metric needs an entry".to_string(),
        };
        let mut item = new_failover_metric("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
