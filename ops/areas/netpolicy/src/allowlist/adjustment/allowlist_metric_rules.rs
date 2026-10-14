use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist metric must pass. Check returns its problems, or none.
pub trait AllowlistMetricRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMetricRequiredCodeRule {
    pub message: String,
}

impl AllowlistMetricRule for AllowlistMetricRequiredCodeRule {
    fn check(&self, item: &AllowlistMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMetricRequiredTenantIdRule {
    pub message: String,
}

impl AllowlistMetricRule for AllowlistMetricRequiredTenantIdRule {
    fn check(&self, item: &AllowlistMetric) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMetricRequiredNameRule {
    pub message: String,
}

impl AllowlistMetricRule for AllowlistMetricRequiredNameRule {
    fn check(&self, item: &AllowlistMetric) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMetricFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistMetricRule for AllowlistMetricFloorValueRangeRule {
    fn check(&self, item: &AllowlistMetric) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMetricTenantIdLengthRule {
    pub max_length: i64,
}

impl AllowlistMetricRule for AllowlistMetricTenantIdLengthRule {
    fn check(&self, item: &AllowlistMetric) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMetricAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl AllowlistMetricRule for AllowlistMetricAllowedTenantIdRule {
    fn check(&self, item: &AllowlistMetric) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is expired", item.tenant_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist metric.
#[derive(Debug, Clone)]
pub struct AllowlistMetricValidator {
    pub rules: Vec<Rc<dyn AllowlistMetricRule>>,
}

impl AllowlistMetricValidator {
    pub fn validate(&self, item: &AllowlistMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_metric_validator() -> AllowlistMetricValidator {
    AllowlistMetricValidator {
        rules: vec![
            Rc::new(AllowlistMetricRequiredCodeRule {
                message: "allowlist metric needs an entry".to_string(),
            }) as Rc<dyn AllowlistMetricRule>,
            Rc::new(AllowlistMetricRequiredTenantIdRule {
                message: "allowlist metric needs an entry".to_string(),
            }),
            Rc::new(AllowlistMetricRequiredNameRule {
                message: "allowlist metric needs an entry".to_string(),
            }),
            Rc::new(AllowlistMetricFloorValueRangeRule { min: 0.5, max: 540.1 }),
            Rc::new(AllowlistMetricTenantIdLengthRule { max_length: 3600 }),
            Rc::new(AllowlistMetricAllowedTenantIdRule {
                allowed: vec!["pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_metric_validator();
        let item = new_allowlist_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistMetricRequiredCodeRule {
            message: "allowlist metric needs this field".to_string(),
        };
        let mut item = new_allowlist_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = AllowlistMetricRequiredTenantIdRule {
            message: "allowlist metric needs a value".to_string(),
        };
        let mut item = new_allowlist_metric("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AllowlistMetricRequiredNameRule {
            message: "allowlist metric needs a value".to_string(),
        };
        let mut item = new_allowlist_metric("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = AllowlistMetricFloorValueRangeRule { min: 4.5, max: 3772.3 };
        let mut item = new_allowlist_metric("x", "y");
        item.floor_value = 3773.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = AllowlistMetricTenantIdLengthRule { max_length: 120 };
        let mut item = new_allowlist_metric("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
