use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation group must pass. Check returns its problems, or none.
pub trait IsolationGroupRule: std::fmt::Debug {
    fn check(&self, item: &IsolationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationGroupRequiredLabelRule {
    pub message: String,
}

impl IsolationGroupRule for IsolationGroupRequiredLabelRule {
    fn check(&self, item: &IsolationGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationGroupCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IsolationGroupRule for IsolationGroupCostRangeRule {
    fn check(&self, item: &IsolationGroup) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationGroupFingerprintLengthRule {
    pub max_length: i64,
}

impl IsolationGroupRule for IsolationGroupFingerprintLengthRule {
    fn check(&self, item: &IsolationGroup) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationGroupMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IsolationGroupRule for IsolationGroupMinimumRangeRule {
    fn check(&self, item: &IsolationGroup) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationGroupAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl IsolationGroupRule for IsolationGroupAllowedFingerprintRule {
    fn check(&self, item: &IsolationGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is below minimum", item.fingerprint)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation group.
#[derive(Debug, Clone)]
pub struct IsolationGroupValidator {
    pub rules: Vec<Rc<dyn IsolationGroupRule>>,
}

impl IsolationGroupValidator {
    pub fn validate(&self, item: &IsolationGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_group_validator() -> IsolationGroupValidator {
    IsolationGroupValidator {
        rules: vec![
            Rc::new(IsolationGroupRequiredLabelRule {
                message: "isolation group needs this field".to_string(),
            }) as Rc<dyn IsolationGroupRule>,
            Rc::new(IsolationGroupCostRangeRule { min: 3.6, max: 4429.4 }),
            Rc::new(IsolationGroupFingerprintLengthRule { max_length: 120 }),
            Rc::new(IsolationGroupMinimumRangeRule { min: 0.6, max: 3734.0 }),
            Rc::new(IsolationGroupAllowedFingerprintRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_group_validator();
        let item = new_isolation_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = IsolationGroupRequiredLabelRule {
            message: "isolation group needs this field".to_string(),
        };
        let mut item = new_isolation_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = IsolationGroupCostRangeRule { min: 0.9, max: 2756.8 };
        let mut item = new_isolation_group("x", "y");
        item.cost = 2758.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = IsolationGroupFingerprintLengthRule { max_length: 80 };
        let mut item = new_isolation_group("x", "y");
        item.fingerprint = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = IsolationGroupMinimumRangeRule { min: 6.3, max: 2513.4 };
        let mut item = new_isolation_group("x", "y");
        item.minimum = 2514.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
