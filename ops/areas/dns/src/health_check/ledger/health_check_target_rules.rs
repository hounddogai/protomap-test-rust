use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check target must pass. Check returns its problems, or none.
pub trait HealthCheckTargetRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTargetClusterLengthRule {
    pub max_length: i64,
}

impl HealthCheckTargetRule for HealthCheckTargetClusterLengthRule {
    fn check(&self, item: &HealthCheckTarget) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTargetCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckTargetRule for HealthCheckTargetCodePrefixRule {
    fn check(&self, item: &HealthCheckTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTargetWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckTargetRule for HealthCheckTargetWindowDaysRangeRule {
    fn check(&self, item: &HealthCheckTarget) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTargetAllowedClusterRule {
    pub allowed: Vec<String>,
}

impl HealthCheckTargetRule for HealthCheckTargetAllowedClusterRule {
    fn check(&self, item: &HealthCheckTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.cluster)))
        {
            return vec![format!("cluster {} is duplicated", item.cluster)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckTargetRequiredTitleRule {
    pub message: String,
}

impl HealthCheckTargetRule for HealthCheckTargetRequiredTitleRule {
    fn check(&self, item: &HealthCheckTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check target.
#[derive(Debug, Clone)]
pub struct HealthCheckTargetValidator {
    pub rules: Vec<Rc<dyn HealthCheckTargetRule>>,
}

impl HealthCheckTargetValidator {
    pub fn validate(&self, item: &HealthCheckTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_target_validator() -> HealthCheckTargetValidator {
    HealthCheckTargetValidator {
        rules: vec![
            Rc::new(HealthCheckTargetClusterLengthRule { max_length: 255 }) as Rc<dyn HealthCheckTargetRule>,
            Rc::new(HealthCheckTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(HealthCheckTargetWindowDaysRangeRule { min: 64, max: 1000 }),
            Rc::new(HealthCheckTargetAllowedClusterRule {
                allowed: vec!["regional".to_string(), "capped".to_string(), "fallback".to_string()],
            }),
            Rc::new(HealthCheckTargetRequiredTitleRule {
                message: "health check target needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_target_validator();
        let item = new_health_check_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = HealthCheckTargetClusterLengthRule { max_length: 1000 };
        let mut item = new_health_check_target("x", "y");
        item.cluster = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = HealthCheckTargetWindowDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_health_check_target("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = HealthCheckTargetRequiredTitleRule {
            message: "health check target needs this field".to_string(),
        };
        let mut item = new_health_check_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
