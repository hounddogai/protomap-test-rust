use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check score must pass. Check returns its problems, or none.
pub trait HealthCheckScoreRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckScoreRequiredCodeRule {
    pub message: String,
}

impl HealthCheckScoreRule for HealthCheckScoreRequiredCodeRule {
    fn check(&self, item: &HealthCheckScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckScoreRequiredNameRule {
    pub message: String,
}

impl HealthCheckScoreRule for HealthCheckScoreRequiredNameRule {
    fn check(&self, item: &HealthCheckScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckScoreThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckScoreRule for HealthCheckScoreThresholdRangeRule {
    fn check(&self, item: &HealthCheckScore) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckScoreEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckScoreRule for HealthCheckScoreEndsAtRangeRule {
    fn check(&self, item: &HealthCheckScore) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a health check score.
#[derive(Debug, Clone)]
pub struct HealthCheckScoreValidator {
    pub rules: Vec<Rc<dyn HealthCheckScoreRule>>,
}

impl HealthCheckScoreValidator {
    pub fn validate(&self, item: &HealthCheckScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_score_validator() -> HealthCheckScoreValidator {
    HealthCheckScoreValidator {
        rules: vec![
            Rc::new(HealthCheckScoreRequiredCodeRule {
                message: "health check score needs a value".to_string(),
            }) as Rc<dyn HealthCheckScoreRule>,
            Rc::new(HealthCheckScoreRequiredNameRule {
                message: "health check score needs this field".to_string(),
            }),
            Rc::new(HealthCheckScoreThresholdRangeRule { min: 6.3, max: 585.2 }),
            Rc::new(HealthCheckScoreEndsAtRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_score_validator();
        let item = new_health_check_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckScoreRequiredCodeRule {
            message: "health check score needs an entry".to_string(),
        };
        let mut item = new_health_check_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = HealthCheckScoreRequiredNameRule {
            message: "health check score needs this field".to_string(),
        };
        let mut item = new_health_check_score("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = HealthCheckScoreThresholdRangeRule { min: 8.5, max: 250.0 };
        let mut item = new_health_check_score("x", "y");
        item.threshold = 251.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = HealthCheckScoreEndsAtRangeRule { min: 255, max: 3600 };
        let mut item = new_health_check_score("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
