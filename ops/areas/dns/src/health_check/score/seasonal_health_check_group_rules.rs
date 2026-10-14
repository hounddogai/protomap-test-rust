use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal health check group must pass. Check returns its problems, or none.
pub trait SeasonalHealthCheckGroupRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalHealthCheckGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckGroupRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalHealthCheckGroupRule for SeasonalHealthCheckGroupRateRangeRule {
    fn check(&self, item: &SeasonalHealthCheckGroup) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckGroupContactEmailLengthRule {
    pub max_length: i64,
}

impl SeasonalHealthCheckGroupRule for SeasonalHealthCheckGroupContactEmailLengthRule {
    fn check(&self, item: &SeasonalHealthCheckGroup) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckGroupRequiredCodeRule {
    pub message: String,
}

impl SeasonalHealthCheckGroupRule for SeasonalHealthCheckGroupRequiredCodeRule {
    fn check(&self, item: &SeasonalHealthCheckGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal health check group.
#[derive(Debug, Clone)]
pub struct SeasonalHealthCheckGroupValidator {
    pub rules: Vec<Rc<dyn SeasonalHealthCheckGroupRule>>,
}

impl SeasonalHealthCheckGroupValidator {
    pub fn validate(&self, item: &SeasonalHealthCheckGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalHealthCheckGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalHealthCheckGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_health_check_group_validator() -> SeasonalHealthCheckGroupValidator {
    SeasonalHealthCheckGroupValidator {
        rules: vec![
            Rc::new(SeasonalHealthCheckGroupRateRangeRule { min: 9.0, max: 3630.1 })
                as Rc<dyn SeasonalHealthCheckGroupRule>,
            Rc::new(SeasonalHealthCheckGroupContactEmailLengthRule { max_length: 32 }),
            Rc::new(SeasonalHealthCheckGroupRequiredCodeRule {
                message: "seasonal health check group needs a value".to_string(),
            }),
        ],
    }
}
