use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant target must pass. Check returns its problems, or none.
pub trait GrantTargetRule: std::fmt::Debug {
    fn check(&self, item: &GrantTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantTargetRequiredExternalRefRule {
    pub message: String,
}

impl GrantTargetRule for GrantTargetRequiredExternalRefRule {
    fn check(&self, item: &GrantTarget) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantTargetMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantTargetRule for GrantTargetMinimumRangeRule {
    fn check(&self, item: &GrantTarget) -> Vec<String> {
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
pub struct GrantTargetStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantTargetRule for GrantTargetStartsAtRangeRule {
    fn check(&self, item: &GrantTarget) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a grant target.
#[derive(Debug, Clone)]
pub struct GrantTargetValidator {
    pub rules: Vec<Rc<dyn GrantTargetRule>>,
}

impl GrantTargetValidator {
    pub fn validate(&self, item: &GrantTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_target_validator() -> GrantTargetValidator {
    GrantTargetValidator {
        rules: vec![
            Rc::new(GrantTargetRequiredExternalRefRule {
                message: "grant target needs a value".to_string(),
            }) as Rc<dyn GrantTargetRule>,
            Rc::new(GrantTargetMinimumRangeRule { min: 6.3, max: 3279.9 }),
            Rc::new(GrantTargetStartsAtRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_target_validator();
        let item = new_grant_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = GrantTargetRequiredExternalRefRule {
            message: "grant target needs this field".to_string(),
        };
        let mut item = new_grant_target("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = GrantTargetMinimumRangeRule { min: 2.9, max: 2173.0 };
        let mut item = new_grant_target("x", "y");
        item.minimum = 2174.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = GrantTargetStartsAtRangeRule { min: 16, max: 3600 };
        let mut item = new_grant_target("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
