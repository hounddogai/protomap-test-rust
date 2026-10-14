use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped dashboard exception must pass. Check returns its problems, or none.
pub trait CappedDashboardExceptionRule: std::fmt::Debug {
    fn check(&self, item: &CappedDashboardException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardExceptionStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedDashboardExceptionRule for CappedDashboardExceptionStartsAtRangeRule {
    fn check(&self, item: &CappedDashboardException) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardExceptionCountryLengthRule {
    pub max_length: i64,
}

impl CappedDashboardExceptionRule for CappedDashboardExceptionCountryLengthRule {
    fn check(&self, item: &CappedDashboardException) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardExceptionChecksumLengthRule {
    pub max_length: i64,
}

impl CappedDashboardExceptionRule for CappedDashboardExceptionChecksumLengthRule {
    fn check(&self, item: &CappedDashboardException) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped dashboard exception.
#[derive(Debug, Clone)]
pub struct CappedDashboardExceptionValidator {
    pub rules: Vec<Rc<dyn CappedDashboardExceptionRule>>,
}

impl CappedDashboardExceptionValidator {
    pub fn validate(&self, item: &CappedDashboardException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedDashboardException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDashboardException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_dashboard_exception_validator() -> CappedDashboardExceptionValidator {
    CappedDashboardExceptionValidator {
        rules: vec![
            Rc::new(CappedDashboardExceptionStartsAtRangeRule { min: 16, max: 64 })
                as Rc<dyn CappedDashboardExceptionRule>,
            Rc::new(CappedDashboardExceptionCountryLengthRule { max_length: 255 }),
            Rc::new(CappedDashboardExceptionChecksumLengthRule { max_length: 80 }),
        ],
    }
}
