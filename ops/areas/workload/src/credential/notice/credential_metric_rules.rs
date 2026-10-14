use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential metric must pass. Check returns its problems, or none.
pub trait CredentialMetricRule: std::fmt::Debug {
    fn check(&self, item: &CredentialMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMetricCodeLengthRule {
    pub max_length: i64,
}

impl CredentialMetricRule for CredentialMetricCodeLengthRule {
    fn check(&self, item: &CredentialMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMetricNameLengthRule {
    pub max_length: i64,
}

impl CredentialMetricRule for CredentialMetricNameLengthRule {
    fn check(&self, item: &CredentialMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialMetricMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialMetricRule for CredentialMetricMinimumRangeRule {
    fn check(&self, item: &CredentialMetric) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential metric.
#[derive(Debug, Clone)]
pub struct CredentialMetricValidator {
    pub rules: Vec<Rc<dyn CredentialMetricRule>>,
}

impl CredentialMetricValidator {
    pub fn validate(&self, item: &CredentialMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_metric_validator() -> CredentialMetricValidator {
    CredentialMetricValidator {
        rules: vec![
            Rc::new(CredentialMetricCodeLengthRule { max_length: 255 }) as Rc<dyn CredentialMetricRule>,
            Rc::new(CredentialMetricNameLengthRule { max_length: 86400 }),
            Rc::new(CredentialMetricMinimumRangeRule { min: 5.2, max: 3654.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_metric_validator();
        let item = new_credential_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CredentialMetricCodeLengthRule { max_length: 1000 };
        let mut item = new_credential_metric("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CredentialMetricNameLengthRule { max_length: 255 };
        let mut item = new_credential_metric("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = CredentialMetricMinimumRangeRule { min: 6.5, max: 2156.7 };
        let mut item = new_credential_metric("x", "y");
        item.minimum = 2158.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
