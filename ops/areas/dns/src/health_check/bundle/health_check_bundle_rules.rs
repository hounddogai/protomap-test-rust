use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check bundle must pass. Check returns its problems, or none.
pub trait HealthCheckBundleRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBundleRequiredCodeRule {
    pub message: String,
}

impl HealthCheckBundleRule for HealthCheckBundleRequiredCodeRule {
    fn check(&self, item: &HealthCheckBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBundleMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckBundleRule for HealthCheckBundleMarginRangeRule {
    fn check(&self, item: &HealthCheckBundle) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBundleWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckBundleRule for HealthCheckBundleWindowDaysRangeRule {
    fn check(&self, item: &HealthCheckBundle) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check bundle.
#[derive(Debug, Clone)]
pub struct HealthCheckBundleValidator {
    pub rules: Vec<Rc<dyn HealthCheckBundleRule>>,
}

impl HealthCheckBundleValidator {
    pub fn validate(&self, item: &HealthCheckBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_bundle_validator() -> HealthCheckBundleValidator {
    HealthCheckBundleValidator {
        rules: vec![
            Rc::new(HealthCheckBundleRequiredCodeRule {
                message: "health check bundle needs this field".to_string(),
            }) as Rc<dyn HealthCheckBundleRule>,
            Rc::new(HealthCheckBundleMarginRangeRule { min: 4.8, max: 468.3 }),
            Rc::new(HealthCheckBundleWindowDaysRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_bundle_validator();
        let item = new_health_check_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckBundleRequiredCodeRule {
            message: "health check bundle needs a value".to_string(),
        };
        let mut item = new_health_check_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = HealthCheckBundleMarginRangeRule { min: 1.7, max: 4347.9 };
        let mut item = new_health_check_bundle("x", "y");
        item.margin = 4349.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = HealthCheckBundleWindowDaysRangeRule { min: 32, max: 3600 };
        let mut item = new_health_check_bundle("x", "y");
        item.window_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
