use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal health check baseline must pass. Check returns its problems, or none.
pub trait SeasonalHealthCheckBaselineRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalHealthCheckBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckBaselineTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalHealthCheckBaselineRule for SeasonalHealthCheckBaselineTitleLengthRule {
    fn check(&self, item: &SeasonalHealthCheckBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckBaselineCodePrefixRule {
    pub prefix: String,
}

impl SeasonalHealthCheckBaselineRule for SeasonalHealthCheckBaselineCodePrefixRule {
    fn check(&self, item: &SeasonalHealthCheckBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckBaselineRequiredTitleRule {
    pub message: String,
}

impl SeasonalHealthCheckBaselineRule for SeasonalHealthCheckBaselineRequiredTitleRule {
    fn check(&self, item: &SeasonalHealthCheckBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckBaselineRequiredCodeRule {
    pub message: String,
}

impl SeasonalHealthCheckBaselineRule for SeasonalHealthCheckBaselineRequiredCodeRule {
    fn check(&self, item: &SeasonalHealthCheckBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckBaselineCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalHealthCheckBaselineRule for SeasonalHealthCheckBaselineCostRangeRule {
    fn check(&self, item: &SeasonalHealthCheckBaseline) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckBaselineCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalHealthCheckBaselineRule for SeasonalHealthCheckBaselineCodeLengthRule {
    fn check(&self, item: &SeasonalHealthCheckBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal health check baseline.
#[derive(Debug, Clone)]
pub struct SeasonalHealthCheckBaselineValidator {
    pub rules: Vec<Rc<dyn SeasonalHealthCheckBaselineRule>>,
}

impl SeasonalHealthCheckBaselineValidator {
    pub fn validate(&self, item: &SeasonalHealthCheckBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalHealthCheckBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalHealthCheckBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_health_check_baseline_validator() -> SeasonalHealthCheckBaselineValidator {
    SeasonalHealthCheckBaselineValidator {
        rules: vec![
            Rc::new(SeasonalHealthCheckBaselineTitleLengthRule { max_length: 32 })
                as Rc<dyn SeasonalHealthCheckBaselineRule>,
            Rc::new(SeasonalHealthCheckBaselineCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SeasonalHealthCheckBaselineRequiredTitleRule {
                message: "seasonal health check baseline needs a value".to_string(),
            }),
            Rc::new(SeasonalHealthCheckBaselineRequiredCodeRule {
                message: "seasonal health check baseline needs this field".to_string(),
            }),
            Rc::new(SeasonalHealthCheckBaselineCostRangeRule { min: 1.7, max: 4004.6 }),
            Rc::new(SeasonalHealthCheckBaselineCodeLengthRule { max_length: 1000 }),
        ],
    }
}
