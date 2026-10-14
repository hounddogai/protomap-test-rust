use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient approval exception must pass. Check returns its problems, or none.
pub trait LenientApprovalExceptionRule: std::fmt::Debug {
    fn check(&self, item: &LenientApprovalException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalExceptionAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientApprovalExceptionRule for LenientApprovalExceptionAmountRangeRule {
    fn check(&self, item: &LenientApprovalException) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalExceptionVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientApprovalExceptionRule for LenientApprovalExceptionVersionRangeRule {
    fn check(&self, item: &LenientApprovalException) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientApprovalExceptionRequiredLabelRule {
    pub message: String,
}

impl LenientApprovalExceptionRule for LenientApprovalExceptionRequiredLabelRule {
    fn check(&self, item: &LenientApprovalException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient approval exception.
#[derive(Debug, Clone)]
pub struct LenientApprovalExceptionValidator {
    pub rules: Vec<Rc<dyn LenientApprovalExceptionRule>>,
}

impl LenientApprovalExceptionValidator {
    pub fn validate(&self, item: &LenientApprovalException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientApprovalException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientApprovalException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_approval_exception_validator() -> LenientApprovalExceptionValidator {
    LenientApprovalExceptionValidator {
        rules: vec![
            Rc::new(LenientApprovalExceptionAmountRangeRule { min: 5.6, max: 3435.7 })
                as Rc<dyn LenientApprovalExceptionRule>,
            Rc::new(LenientApprovalExceptionVersionRangeRule { min: 64, max: 3600 }),
            Rc::new(LenientApprovalExceptionRequiredLabelRule {
                message: "lenient approval exception needs a value".to_string(),
            }),
        ],
    }
}
