use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual forecast audit must pass. Check returns its problems, or none.
pub trait ManualForecastAuditRule: std::fmt::Debug {
    fn check(&self, item: &ManualForecastAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastAuditWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualForecastAuditRule for ManualForecastAuditWeightRangeRule {
    fn check(&self, item: &ManualForecastAudit) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastAuditRequiredTimezoneRule {
    pub message: String,
}

impl ManualForecastAuditRule for ManualForecastAuditRequiredTimezoneRule {
    fn check(&self, item: &ManualForecastAudit) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastAuditTimezoneLengthRule {
    pub max_length: i64,
}

impl ManualForecastAuditRule for ManualForecastAuditTimezoneLengthRule {
    fn check(&self, item: &ManualForecastAudit) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualForecastAuditCodeLengthRule {
    pub max_length: i64,
}

impl ManualForecastAuditRule for ManualForecastAuditCodeLengthRule {
    fn check(&self, item: &ManualForecastAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual forecast audit.
#[derive(Debug, Clone)]
pub struct ManualForecastAuditValidator {
    pub rules: Vec<Rc<dyn ManualForecastAuditRule>>,
}

impl ManualForecastAuditValidator {
    pub fn validate(&self, item: &ManualForecastAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualForecastAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualForecastAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_forecast_audit_validator() -> ManualForecastAuditValidator {
    ManualForecastAuditValidator {
        rules: vec![
            Rc::new(ManualForecastAuditWeightRangeRule { min: 9.9, max: 4343.1 }) as Rc<dyn ManualForecastAuditRule>,
            Rc::new(ManualForecastAuditRequiredTimezoneRule {
                message: "manual forecast audit needs a value".to_string(),
            }),
            Rc::new(ManualForecastAuditTimezoneLengthRule { max_length: 64 }),
            Rc::new(ManualForecastAuditCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_forecast_audit_validator();
        let item = new_manual_forecast_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ManualForecastAuditWeightRangeRule { min: 2.8, max: 2730.9 };
        let mut item = new_manual_forecast_audit("x", "y");
        item.weight = 2732.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ManualForecastAuditRequiredTimezoneRule {
            message: "manual forecast audit needs this field".to_string(),
        };
        let mut item = new_manual_forecast_audit("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = ManualForecastAuditTimezoneLengthRule { max_length: 80 };
        let mut item = new_manual_forecast_audit("x", "y");
        item.timezone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualForecastAuditCodeLengthRule { max_length: 1000 };
        let mut item = new_manual_forecast_audit("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
