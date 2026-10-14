use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered postmortem digest must pass. Check returns its problems, or none.
pub trait TieredPostmortemDigestRule: std::fmt::Debug {
    fn check(&self, item: &TieredPostmortemDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemDigestRequiredCodeRule {
    pub message: String,
}

impl TieredPostmortemDigestRule for TieredPostmortemDigestRequiredCodeRule {
    fn check(&self, item: &TieredPostmortemDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemDigestNameLengthRule {
    pub max_length: i64,
}

impl TieredPostmortemDigestRule for TieredPostmortemDigestNameLengthRule {
    fn check(&self, item: &TieredPostmortemDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemDigestCodeLengthRule {
    pub max_length: i64,
}

impl TieredPostmortemDigestRule for TieredPostmortemDigestCodeLengthRule {
    fn check(&self, item: &TieredPostmortemDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemDigestRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredPostmortemDigestRule for TieredPostmortemDigestRotationDaysRangeRule {
    fn check(&self, item: &TieredPostmortemDigest) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered postmortem digest.
#[derive(Debug, Clone)]
pub struct TieredPostmortemDigestValidator {
    pub rules: Vec<Rc<dyn TieredPostmortemDigestRule>>,
}

impl TieredPostmortemDigestValidator {
    pub fn validate(&self, item: &TieredPostmortemDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredPostmortemDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPostmortemDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_postmortem_digest_validator() -> TieredPostmortemDigestValidator {
    TieredPostmortemDigestValidator {
        rules: vec![
            Rc::new(TieredPostmortemDigestRequiredCodeRule {
                message: "tiered postmortem digest needs this field".to_string(),
            }) as Rc<dyn TieredPostmortemDigestRule>,
            Rc::new(TieredPostmortemDigestNameLengthRule { max_length: 80 }),
            Rc::new(TieredPostmortemDigestCodeLengthRule { max_length: 120 }),
            Rc::new(TieredPostmortemDigestRotationDaysRangeRule { min: 120, max: 1000 }),
        ],
    }
}
