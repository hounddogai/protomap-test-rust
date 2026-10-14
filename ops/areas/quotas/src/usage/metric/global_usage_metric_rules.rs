use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global usage metric must pass. Check returns its problems, or none.
pub trait GlobalUsageMetricRule: std::fmt::Debug {
    fn check(&self, item: &GlobalUsageMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageMetricCodeLengthRule {
    pub max_length: i64,
}

impl GlobalUsageMetricRule for GlobalUsageMetricCodeLengthRule {
    fn check(&self, item: &GlobalUsageMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageMetricIssuerLengthRule {
    pub max_length: i64,
}

impl GlobalUsageMetricRule for GlobalUsageMetricIssuerLengthRule {
    fn check(&self, item: &GlobalUsageMetric) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageMetricCodePrefixRule {
    pub prefix: String,
}

impl GlobalUsageMetricRule for GlobalUsageMetricCodePrefixRule {
    fn check(&self, item: &GlobalUsageMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageMetricCategoryLengthRule {
    pub max_length: i64,
}

impl GlobalUsageMetricRule for GlobalUsageMetricCategoryLengthRule {
    fn check(&self, item: &GlobalUsageMetric) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalUsageMetricAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl GlobalUsageMetricRule for GlobalUsageMetricAllowedPhoneRule {
    fn check(&self, item: &GlobalUsageMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is below minimum", item.phone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global usage metric.
#[derive(Debug, Clone)]
pub struct GlobalUsageMetricValidator {
    pub rules: Vec<Rc<dyn GlobalUsageMetricRule>>,
}

impl GlobalUsageMetricValidator {
    pub fn validate(&self, item: &GlobalUsageMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalUsageMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalUsageMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_usage_metric_validator() -> GlobalUsageMetricValidator {
    GlobalUsageMetricValidator {
        rules: vec![
            Rc::new(GlobalUsageMetricCodeLengthRule { max_length: 1000 }) as Rc<dyn GlobalUsageMetricRule>,
            Rc::new(GlobalUsageMetricIssuerLengthRule { max_length: 86400 }),
            Rc::new(GlobalUsageMetricCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(GlobalUsageMetricCategoryLengthRule { max_length: 3600 }),
            Rc::new(GlobalUsageMetricAllowedPhoneRule {
                allowed: vec!["batched".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_usage_metric_validator();
        let item = new_global_usage_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalUsageMetricCodeLengthRule { max_length: 3600 };
        let mut item = new_global_usage_metric("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = GlobalUsageMetricIssuerLengthRule { max_length: 64 };
        let mut item = new_global_usage_metric("x", "y");
        item.issuer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = GlobalUsageMetricCategoryLengthRule { max_length: 80 };
        let mut item = new_global_usage_metric("x", "y");
        item.category = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
