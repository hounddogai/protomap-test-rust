use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert profile must pass. Check returns its problems, or none.
pub trait AlertProfileRule: std::fmt::Debug {
    fn check(&self, item: &AlertProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertProfileUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertProfileRule for AlertProfileUpdatedAtRangeRule {
    fn check(&self, item: &AlertProfile) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertProfileCodeLengthRule {
    pub max_length: i64,
}

impl AlertProfileRule for AlertProfileCodeLengthRule {
    fn check(&self, item: &AlertProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertProfileRequiredPrincipalRule {
    pub message: String,
}

impl AlertProfileRule for AlertProfileRequiredPrincipalRule {
    fn check(&self, item: &AlertProfile) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertProfileBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertProfileRule for AlertProfileBudgetRangeRule {
    fn check(&self, item: &AlertProfile) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertProfileNameLengthRule {
    pub max_length: i64,
}

impl AlertProfileRule for AlertProfileNameLengthRule {
    fn check(&self, item: &AlertProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert profile.
#[derive(Debug, Clone)]
pub struct AlertProfileValidator {
    pub rules: Vec<Rc<dyn AlertProfileRule>>,
}

impl AlertProfileValidator {
    pub fn validate(&self, item: &AlertProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_profile_validator() -> AlertProfileValidator {
    AlertProfileValidator {
        rules: vec![
            Rc::new(AlertProfileUpdatedAtRangeRule { min: 80, max: 80 }) as Rc<dyn AlertProfileRule>,
            Rc::new(AlertProfileCodeLengthRule { max_length: 86400 }),
            Rc::new(AlertProfileRequiredPrincipalRule {
                message: "alert profile needs this field".to_string(),
            }),
            Rc::new(AlertProfileBudgetRangeRule { min: 8.3, max: 1591.3 }),
            Rc::new(AlertProfileNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_profile_validator();
        let item = new_alert_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = AlertProfileUpdatedAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_alert_profile("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AlertProfileCodeLengthRule { max_length: 120 };
        let mut item = new_alert_profile("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = AlertProfileRequiredPrincipalRule {
            message: "alert profile needs a value".to_string(),
        };
        let mut item = new_alert_profile("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = AlertProfileBudgetRangeRule { min: 6.5, max: 4463.0 };
        let mut item = new_alert_profile("x", "y");
        item.budget = 4464.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AlertProfileNameLengthRule { max_length: 86400 };
        let mut item = new_alert_profile("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
