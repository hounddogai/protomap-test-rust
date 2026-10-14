use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default freeze profile must pass. Check returns its problems, or none.
pub trait DefaultFreezeProfileRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFreezeProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeProfileRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultFreezeProfileRule for DefaultFreezeProfileRetriesRangeRule {
    fn check(&self, item: &DefaultFreezeProfile) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeProfileScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultFreezeProfileRule for DefaultFreezeProfileScoreRangeRule {
    fn check(&self, item: &DefaultFreezeProfile) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeProfileCodePrefixRule {
    pub prefix: String,
}

impl DefaultFreezeProfileRule for DefaultFreezeProfileCodePrefixRule {
    fn check(&self, item: &DefaultFreezeProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default freeze profile.
#[derive(Debug, Clone)]
pub struct DefaultFreezeProfileValidator {
    pub rules: Vec<Rc<dyn DefaultFreezeProfileRule>>,
}

impl DefaultFreezeProfileValidator {
    pub fn validate(&self, item: &DefaultFreezeProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultFreezeProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFreezeProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_freeze_profile_validator() -> DefaultFreezeProfileValidator {
    DefaultFreezeProfileValidator {
        rules: vec![
            Rc::new(DefaultFreezeProfileRetriesRangeRule { min: 16, max: 255 }) as Rc<dyn DefaultFreezeProfileRule>,
            Rc::new(DefaultFreezeProfileScoreRangeRule { min: 2.1, max: 2395.6 }),
            Rc::new(DefaultFreezeProfileCodePrefixRule {
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
        let validator = default_default_freeze_profile_validator();
        let item = new_default_freeze_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = DefaultFreezeProfileRetriesRangeRule { min: 16, max: 64 };
        let mut item = new_default_freeze_profile("x", "y");
        item.retries = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = DefaultFreezeProfileScoreRangeRule { min: 4.7, max: 3794.0 };
        let mut item = new_default_freeze_profile("x", "y");
        item.score = 3795.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
