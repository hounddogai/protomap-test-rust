use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal canary exception must pass. Check returns its problems, or none.
pub trait SeasonalCanaryExceptionRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalCanaryException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryExceptionBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalCanaryExceptionRule for SeasonalCanaryExceptionBurstRangeRule {
    fn check(&self, item: &SeasonalCanaryException) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryExceptionCodePrefixRule {
    pub prefix: String,
}

impl SeasonalCanaryExceptionRule for SeasonalCanaryExceptionCodePrefixRule {
    fn check(&self, item: &SeasonalCanaryException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryExceptionDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalCanaryExceptionRule for SeasonalCanaryExceptionDiscountRangeRule {
    fn check(&self, item: &SeasonalCanaryException) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal canary exception.
#[derive(Debug, Clone)]
pub struct SeasonalCanaryExceptionValidator {
    pub rules: Vec<Rc<dyn SeasonalCanaryExceptionRule>>,
}

impl SeasonalCanaryExceptionValidator {
    pub fn validate(&self, item: &SeasonalCanaryException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalCanaryException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalCanaryException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_canary_exception_validator() -> SeasonalCanaryExceptionValidator {
    SeasonalCanaryExceptionValidator {
        rules: vec![
            Rc::new(SeasonalCanaryExceptionBurstRangeRule { min: 80, max: 255 }) as Rc<dyn SeasonalCanaryExceptionRule>,
            Rc::new(SeasonalCanaryExceptionCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SeasonalCanaryExceptionDiscountRangeRule { min: 4.7, max: 4987.6 }),
        ],
    }
}
