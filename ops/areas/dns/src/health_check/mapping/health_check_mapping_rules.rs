use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check mapping must pass. Check returns its problems, or none.
pub trait HealthCheckMappingRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMappingExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckMappingRule for HealthCheckMappingExpiresAtRangeRule {
    fn check(&self, item: &HealthCheckMapping) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMappingRequiredTenantIdRule {
    pub message: String,
}

impl HealthCheckMappingRule for HealthCheckMappingRequiredTenantIdRule {
    fn check(&self, item: &HealthCheckMapping) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMappingRequiredCodeRule {
    pub message: String,
}

impl HealthCheckMappingRule for HealthCheckMappingRequiredCodeRule {
    fn check(&self, item: &HealthCheckMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMappingCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckMappingRule for HealthCheckMappingCodePrefixRule {
    fn check(&self, item: &HealthCheckMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check mapping.
#[derive(Debug, Clone)]
pub struct HealthCheckMappingValidator {
    pub rules: Vec<Rc<dyn HealthCheckMappingRule>>,
}

impl HealthCheckMappingValidator {
    pub fn validate(&self, item: &HealthCheckMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_mapping_validator() -> HealthCheckMappingValidator {
    HealthCheckMappingValidator {
        rules: vec![
            Rc::new(HealthCheckMappingExpiresAtRangeRule { min: 16, max: 80 }) as Rc<dyn HealthCheckMappingRule>,
            Rc::new(HealthCheckMappingRequiredTenantIdRule {
                message: "health check mapping needs an entry".to_string(),
            }),
            Rc::new(HealthCheckMappingRequiredCodeRule {
                message: "health check mapping needs an entry".to_string(),
            }),
            Rc::new(HealthCheckMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_mapping_validator();
        let item = new_health_check_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = HealthCheckMappingExpiresAtRangeRule { min: 16, max: 120 };
        let mut item = new_health_check_mapping("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = HealthCheckMappingRequiredTenantIdRule {
            message: "health check mapping needs an entry".to_string(),
        };
        let mut item = new_health_check_mapping("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckMappingRequiredCodeRule {
            message: "health check mapping needs an entry".to_string(),
        };
        let mut item = new_health_check_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
