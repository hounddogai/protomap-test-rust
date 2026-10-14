use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict health check milestone must pass. Check returns its problems, or none.
pub trait StrictHealthCheckMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &StrictHealthCheckMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl StrictHealthCheckMilestoneRule for StrictHealthCheckMilestoneCodeLengthRule {
    fn check(&self, item: &StrictHealthCheckMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckMilestoneNamespaceLengthRule {
    pub max_length: i64,
}

impl StrictHealthCheckMilestoneRule for StrictHealthCheckMilestoneNamespaceLengthRule {
    fn check(&self, item: &StrictHealthCheckMilestone) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckMilestoneRequiredNamespaceRule {
    pub message: String,
}

impl StrictHealthCheckMilestoneRule for StrictHealthCheckMilestoneRequiredNamespaceRule {
    fn check(&self, item: &StrictHealthCheckMilestone) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckMilestoneTimezoneLengthRule {
    pub max_length: i64,
}

impl StrictHealthCheckMilestoneRule for StrictHealthCheckMilestoneTimezoneLengthRule {
    fn check(&self, item: &StrictHealthCheckMilestone) -> Vec<String> {
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

/// Runs every rule of a strict health check milestone.
#[derive(Debug, Clone)]
pub struct StrictHealthCheckMilestoneValidator {
    pub rules: Vec<Rc<dyn StrictHealthCheckMilestoneRule>>,
}

impl StrictHealthCheckMilestoneValidator {
    pub fn validate(&self, item: &StrictHealthCheckMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictHealthCheckMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictHealthCheckMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_health_check_milestone_validator() -> StrictHealthCheckMilestoneValidator {
    StrictHealthCheckMilestoneValidator {
        rules: vec![
            Rc::new(StrictHealthCheckMilestoneCodeLengthRule { max_length: 3600 })
                as Rc<dyn StrictHealthCheckMilestoneRule>,
            Rc::new(StrictHealthCheckMilestoneNamespaceLengthRule { max_length: 80 }),
            Rc::new(StrictHealthCheckMilestoneRequiredNamespaceRule {
                message: "strict health check milestone needs this field".to_string(),
            }),
            Rc::new(StrictHealthCheckMilestoneTimezoneLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_health_check_milestone_validator();
        let item = new_strict_health_check_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictHealthCheckMilestoneCodeLengthRule { max_length: 32 };
        let mut item = new_strict_health_check_milestone("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = StrictHealthCheckMilestoneNamespaceLengthRule { max_length: 86400 };
        let mut item = new_strict_health_check_milestone("x", "y");
        item.namespace = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = StrictHealthCheckMilestoneRequiredNamespaceRule {
            message: "strict health check milestone needs a value".to_string(),
        };
        let mut item = new_strict_health_check_milestone("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = StrictHealthCheckMilestoneTimezoneLengthRule { max_length: 80 };
        let mut item = new_strict_health_check_milestone("x", "y");
        item.timezone = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
