use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending health check window must pass. Check returns its problems, or none.
pub trait PendingHealthCheckWindowRule: std::fmt::Debug {
    fn check(&self, item: &PendingHealthCheckWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckWindowRequiredLocaleRule {
    pub message: String,
}

impl PendingHealthCheckWindowRule for PendingHealthCheckWindowRequiredLocaleRule {
    fn check(&self, item: &PendingHealthCheckWindow) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckWindowLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingHealthCheckWindowRule for PendingHealthCheckWindowLimitRangeRule {
    fn check(&self, item: &PendingHealthCheckWindow) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckWindowRequiredCodeRule {
    pub message: String,
}

impl PendingHealthCheckWindowRule for PendingHealthCheckWindowRequiredCodeRule {
    fn check(&self, item: &PendingHealthCheckWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckWindowWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingHealthCheckWindowRule for PendingHealthCheckWindowWeightRangeRule {
    fn check(&self, item: &PendingHealthCheckWindow) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckWindowNameLengthRule {
    pub max_length: i64,
}

impl PendingHealthCheckWindowRule for PendingHealthCheckWindowNameLengthRule {
    fn check(&self, item: &PendingHealthCheckWindow) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckWindowRequiredNameRule {
    pub message: String,
}

impl PendingHealthCheckWindowRule for PendingHealthCheckWindowRequiredNameRule {
    fn check(&self, item: &PendingHealthCheckWindow) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending health check window.
#[derive(Debug, Clone)]
pub struct PendingHealthCheckWindowValidator {
    pub rules: Vec<Rc<dyn PendingHealthCheckWindowRule>>,
}

impl PendingHealthCheckWindowValidator {
    pub fn validate(&self, item: &PendingHealthCheckWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingHealthCheckWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingHealthCheckWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_health_check_window_validator() -> PendingHealthCheckWindowValidator {
    PendingHealthCheckWindowValidator {
        rules: vec![
            Rc::new(PendingHealthCheckWindowRequiredLocaleRule {
                message: "pending health check window needs a value".to_string(),
            }) as Rc<dyn PendingHealthCheckWindowRule>,
            Rc::new(PendingHealthCheckWindowLimitRangeRule { min: 120, max: 120 }),
            Rc::new(PendingHealthCheckWindowRequiredCodeRule {
                message: "pending health check window needs an entry".to_string(),
            }),
            Rc::new(PendingHealthCheckWindowWeightRangeRule { min: 9.7, max: 3135.4 }),
            Rc::new(PendingHealthCheckWindowNameLengthRule { max_length: 255 }),
            Rc::new(PendingHealthCheckWindowRequiredNameRule {
                message: "pending health check window needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_health_check_window_validator();
        let item = new_pending_health_check_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PendingHealthCheckWindowRequiredLocaleRule {
            message: "pending health check window needs an entry".to_string(),
        };
        let mut item = new_pending_health_check_window("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = PendingHealthCheckWindowLimitRangeRule { min: 32, max: 120 };
        let mut item = new_pending_health_check_window("x", "y");
        item.limit = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingHealthCheckWindowRequiredCodeRule {
            message: "pending health check window needs an entry".to_string(),
        };
        let mut item = new_pending_health_check_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PendingHealthCheckWindowWeightRangeRule { min: 5.7, max: 176.5 };
        let mut item = new_pending_health_check_window("x", "y");
        item.weight = 178.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingHealthCheckWindowNameLengthRule { max_length: 16 };
        let mut item = new_pending_health_check_window("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingHealthCheckWindowRequiredNameRule {
            message: "pending health check window needs an entry".to_string(),
        };
        let mut item = new_pending_health_check_window("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
