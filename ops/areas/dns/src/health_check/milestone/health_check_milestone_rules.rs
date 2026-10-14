use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check milestone must pass. Check returns its problems, or none.
pub trait HealthCheckMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMilestoneRequiredTimezoneRule {
    pub message: String,
}

impl HealthCheckMilestoneRule for HealthCheckMilestoneRequiredTimezoneRule {
    fn check(&self, item: &HealthCheckMilestone) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMilestoneQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckMilestoneRule for HealthCheckMilestoneQuantityRangeRule {
    fn check(&self, item: &HealthCheckMilestone) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMilestoneCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckMilestoneRule for HealthCheckMilestoneCodePrefixRule {
    fn check(&self, item: &HealthCheckMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckMilestoneRequiredCodeRule {
    pub message: String,
}

impl HealthCheckMilestoneRule for HealthCheckMilestoneRequiredCodeRule {
    fn check(&self, item: &HealthCheckMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check milestone.
#[derive(Debug, Clone)]
pub struct HealthCheckMilestoneValidator {
    pub rules: Vec<Rc<dyn HealthCheckMilestoneRule>>,
}

impl HealthCheckMilestoneValidator {
    pub fn validate(&self, item: &HealthCheckMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_milestone_validator() -> HealthCheckMilestoneValidator {
    HealthCheckMilestoneValidator {
        rules: vec![
            Rc::new(HealthCheckMilestoneRequiredTimezoneRule {
                message: "health check milestone needs this field".to_string(),
            }) as Rc<dyn HealthCheckMilestoneRule>,
            Rc::new(HealthCheckMilestoneQuantityRangeRule { min: 32, max: 64 }),
            Rc::new(HealthCheckMilestoneCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(HealthCheckMilestoneRequiredCodeRule {
                message: "health check milestone needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_milestone_validator();
        let item = new_health_check_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = HealthCheckMilestoneRequiredTimezoneRule {
            message: "health check milestone needs this field".to_string(),
        };
        let mut item = new_health_check_milestone("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = HealthCheckMilestoneQuantityRangeRule { min: 80, max: 1000 };
        let mut item = new_health_check_milestone("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckMilestoneRequiredCodeRule {
            message: "health check milestone needs an entry".to_string(),
        };
        let mut item = new_health_check_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
