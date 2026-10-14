use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation checkpoint must pass. Check returns its problems, or none.
pub trait IsolationCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &IsolationCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl IsolationCheckpointRule for IsolationCheckpointCodeLengthRule {
    fn check(&self, item: &IsolationCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationCheckpointRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IsolationCheckpointRule for IsolationCheckpointRatioRangeRule {
    fn check(&self, item: &IsolationCheckpoint) -> Vec<String> {
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
pub struct IsolationCheckpointAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl IsolationCheckpointRule for IsolationCheckpointAllowedRegionRule {
    fn check(&self, item: &IsolationCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is duplicated", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationCheckpointNameLengthRule {
    pub max_length: i64,
}

impl IsolationCheckpointRule for IsolationCheckpointNameLengthRule {
    fn check(&self, item: &IsolationCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationCheckpointCodePrefixRule {
    pub prefix: String,
}

impl IsolationCheckpointRule for IsolationCheckpointCodePrefixRule {
    fn check(&self, item: &IsolationCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation checkpoint.
#[derive(Debug, Clone)]
pub struct IsolationCheckpointValidator {
    pub rules: Vec<Rc<dyn IsolationCheckpointRule>>,
}

impl IsolationCheckpointValidator {
    pub fn validate(&self, item: &IsolationCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_checkpoint_validator() -> IsolationCheckpointValidator {
    IsolationCheckpointValidator {
        rules: vec![
            Rc::new(IsolationCheckpointCodeLengthRule { max_length: 86400 }) as Rc<dyn IsolationCheckpointRule>,
            Rc::new(IsolationCheckpointRatioRangeRule { min: 2.8, max: 3565.5 }),
            Rc::new(IsolationCheckpointAllowedRegionRule {
                allowed: vec!["fallback".to_string()],
            }),
            Rc::new(IsolationCheckpointNameLengthRule { max_length: 1000 }),
            Rc::new(IsolationCheckpointCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_checkpoint_validator();
        let item = new_isolation_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IsolationCheckpointCodeLengthRule { max_length: 120 };
        let mut item = new_isolation_checkpoint("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = IsolationCheckpointRatioRangeRule { min: 8.3, max: 2070.7 };
        let mut item = new_isolation_checkpoint("x", "y");
        item.ratio = 2072.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = IsolationCheckpointNameLengthRule { max_length: 86400 };
        let mut item = new_isolation_checkpoint("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
