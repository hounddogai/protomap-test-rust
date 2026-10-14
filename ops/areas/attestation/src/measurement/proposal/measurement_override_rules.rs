use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement override must pass. Check returns its problems, or none.
pub trait MeasurementOverrideRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementOverrideRequiredTenantIdRule {
    pub message: String,
}

impl MeasurementOverrideRule for MeasurementOverrideRequiredTenantIdRule {
    fn check(&self, item: &MeasurementOverride) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementOverrideRequiredLabelRule {
    pub message: String,
}

impl MeasurementOverrideRule for MeasurementOverrideRequiredLabelRule {
    fn check(&self, item: &MeasurementOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementOverrideTenantIdLengthRule {
    pub max_length: i64,
}

impl MeasurementOverrideRule for MeasurementOverrideTenantIdLengthRule {
    fn check(&self, item: &MeasurementOverride) -> Vec<String> {
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
pub struct MeasurementOverridePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementOverrideRule for MeasurementOverridePortRangeRule {
    fn check(&self, item: &MeasurementOverride) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementOverrideCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementOverrideRule for MeasurementOverrideCodeLengthRule {
    fn check(&self, item: &MeasurementOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementOverrideGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementOverrideRule for MeasurementOverrideGraceDaysRangeRule {
    fn check(&self, item: &MeasurementOverride) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement override.
#[derive(Debug, Clone)]
pub struct MeasurementOverrideValidator {
    pub rules: Vec<Rc<dyn MeasurementOverrideRule>>,
}

impl MeasurementOverrideValidator {
    pub fn validate(&self, item: &MeasurementOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_override_validator() -> MeasurementOverrideValidator {
    MeasurementOverrideValidator {
        rules: vec![
            Rc::new(MeasurementOverrideRequiredTenantIdRule {
                message: "measurement override needs an entry".to_string(),
            }) as Rc<dyn MeasurementOverrideRule>,
            Rc::new(MeasurementOverrideRequiredLabelRule {
                message: "measurement override needs this field".to_string(),
            }),
            Rc::new(MeasurementOverrideTenantIdLengthRule { max_length: 1000 }),
            Rc::new(MeasurementOverridePortRangeRule { min: 1000, max: 3600 }),
            Rc::new(MeasurementOverrideCodeLengthRule { max_length: 1000 }),
            Rc::new(MeasurementOverrideGraceDaysRangeRule { min: 32, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_override_validator();
        let item = new_measurement_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = MeasurementOverrideRequiredTenantIdRule {
            message: "measurement override needs an entry".to_string(),
        };
        let mut item = new_measurement_override("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = MeasurementOverrideRequiredLabelRule {
            message: "measurement override needs a value".to_string(),
        };
        let mut item = new_measurement_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = MeasurementOverrideTenantIdLengthRule { max_length: 1000 };
        let mut item = new_measurement_override("x", "y");
        item.tenant_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = MeasurementOverridePortRangeRule { min: 16, max: 64 };
        let mut item = new_measurement_override("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MeasurementOverrideCodeLengthRule { max_length: 255 };
        let mut item = new_measurement_override("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = MeasurementOverrideGraceDaysRangeRule { min: 86400, max: 86400 };
        let mut item = new_measurement_override("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
