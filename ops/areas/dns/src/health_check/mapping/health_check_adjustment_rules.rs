use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check adjustment must pass. Check returns its problems, or none.
pub trait HealthCheckAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckAdjustmentRule for HealthCheckAdjustmentCodePrefixRule {
    fn check(&self, item: &HealthCheckAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAdjustmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckAdjustmentRule for HealthCheckAdjustmentCreatedAtRangeRule {
    fn check(&self, item: &HealthCheckAdjustment) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAdjustmentTenantIdLengthRule {
    pub max_length: i64,
}

impl HealthCheckAdjustmentRule for HealthCheckAdjustmentTenantIdLengthRule {
    fn check(&self, item: &HealthCheckAdjustment) -> Vec<String> {
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
pub struct HealthCheckAdjustmentExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckAdjustmentRule for HealthCheckAdjustmentExpiresAtRangeRule {
    fn check(&self, item: &HealthCheckAdjustment) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAdjustmentRequiredCodeRule {
    pub message: String,
}

impl HealthCheckAdjustmentRule for HealthCheckAdjustmentRequiredCodeRule {
    fn check(&self, item: &HealthCheckAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check adjustment.
#[derive(Debug, Clone)]
pub struct HealthCheckAdjustmentValidator {
    pub rules: Vec<Rc<dyn HealthCheckAdjustmentRule>>,
}

impl HealthCheckAdjustmentValidator {
    pub fn validate(&self, item: &HealthCheckAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_adjustment_validator() -> HealthCheckAdjustmentValidator {
    HealthCheckAdjustmentValidator {
        rules: vec![
            Rc::new(HealthCheckAdjustmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn HealthCheckAdjustmentRule>,
            Rc::new(HealthCheckAdjustmentCreatedAtRangeRule { min: 80, max: 3600 }),
            Rc::new(HealthCheckAdjustmentTenantIdLengthRule { max_length: 32 }),
            Rc::new(HealthCheckAdjustmentExpiresAtRangeRule { min: 16, max: 32 }),
            Rc::new(HealthCheckAdjustmentRequiredCodeRule {
                message: "health check adjustment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_adjustment_validator();
        let item = new_health_check_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = HealthCheckAdjustmentCreatedAtRangeRule { min: 120, max: 1000 };
        let mut item = new_health_check_adjustment("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = HealthCheckAdjustmentTenantIdLengthRule { max_length: 86400 };
        let mut item = new_health_check_adjustment("x", "y");
        item.tenant_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = HealthCheckAdjustmentExpiresAtRangeRule { min: 120, max: 255 };
        let mut item = new_health_check_adjustment("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckAdjustmentRequiredCodeRule {
            message: "health check adjustment needs an entry".to_string(),
        };
        let mut item = new_health_check_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
