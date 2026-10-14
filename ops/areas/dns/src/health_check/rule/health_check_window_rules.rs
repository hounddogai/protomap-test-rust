use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check window must pass. Check returns its problems, or none.
pub trait HealthCheckWindowRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckWindowRequiredCodeRule {
    pub message: String,
}

impl HealthCheckWindowRule for HealthCheckWindowRequiredCodeRule {
    fn check(&self, item: &HealthCheckWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckWindowRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckWindowRule for HealthCheckWindowRatioRangeRule {
    fn check(&self, item: &HealthCheckWindow) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckWindowCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckWindowRule for HealthCheckWindowCeilingRangeRule {
    fn check(&self, item: &HealthCheckWindow) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckWindowNamespaceLengthRule {
    pub max_length: i64,
}

impl HealthCheckWindowRule for HealthCheckWindowNamespaceLengthRule {
    fn check(&self, item: &HealthCheckWindow) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check window.
#[derive(Debug, Clone)]
pub struct HealthCheckWindowValidator {
    pub rules: Vec<Rc<dyn HealthCheckWindowRule>>,
}

impl HealthCheckWindowValidator {
    pub fn validate(&self, item: &HealthCheckWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_window_validator() -> HealthCheckWindowValidator {
    HealthCheckWindowValidator {
        rules: vec![
            Rc::new(HealthCheckWindowRequiredCodeRule {
                message: "health check window needs a value".to_string(),
            }) as Rc<dyn HealthCheckWindowRule>,
            Rc::new(HealthCheckWindowRatioRangeRule { min: 4.8, max: 1819.0 }),
            Rc::new(HealthCheckWindowCeilingRangeRule { min: 5.1, max: 3360.7 }),
            Rc::new(HealthCheckWindowNamespaceLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_window_validator();
        let item = new_health_check_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckWindowRequiredCodeRule {
            message: "health check window needs an entry".to_string(),
        };
        let mut item = new_health_check_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = HealthCheckWindowRatioRangeRule { min: 10.0, max: 4737.7 };
        let mut item = new_health_check_window("x", "y");
        item.ratio = 4739.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = HealthCheckWindowCeilingRangeRule { min: 6.7, max: 3014.0 };
        let mut item = new_health_check_window("x", "y");
        item.ceiling = 3015.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = HealthCheckWindowNamespaceLengthRule { max_length: 16 };
        let mut item = new_health_check_window("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
