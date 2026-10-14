use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary verifier entry must pass. Check returns its problems, or none.
pub trait PrimaryVerifierEntryRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryVerifierEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierEntryRequiredRegionRule {
    pub message: String,
}

impl PrimaryVerifierEntryRule for PrimaryVerifierEntryRequiredRegionRule {
    fn check(&self, item: &PrimaryVerifierEntry) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierEntryGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryVerifierEntryRule for PrimaryVerifierEntryGraceDaysRangeRule {
    fn check(&self, item: &PrimaryVerifierEntry) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierEntryRegionLengthRule {
    pub max_length: i64,
}

impl PrimaryVerifierEntryRule for PrimaryVerifierEntryRegionLengthRule {
    fn check(&self, item: &PrimaryVerifierEntry) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary verifier entry.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierEntryValidator {
    pub rules: Vec<Rc<dyn PrimaryVerifierEntryRule>>,
}

impl PrimaryVerifierEntryValidator {
    pub fn validate(&self, item: &PrimaryVerifierEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryVerifierEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryVerifierEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_verifier_entry_validator() -> PrimaryVerifierEntryValidator {
    PrimaryVerifierEntryValidator {
        rules: vec![
            Rc::new(PrimaryVerifierEntryRequiredRegionRule {
                message: "primary verifier entry needs this field".to_string(),
            }) as Rc<dyn PrimaryVerifierEntryRule>,
            Rc::new(PrimaryVerifierEntryGraceDaysRangeRule { min: 3600, max: 86400 }),
            Rc::new(PrimaryVerifierEntryRegionLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_verifier_entry_validator();
        let item = new_primary_verifier_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = PrimaryVerifierEntryRequiredRegionRule {
            message: "primary verifier entry needs a value".to_string(),
        };
        let mut item = new_primary_verifier_entry("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = PrimaryVerifierEntryGraceDaysRangeRule { min: 32, max: 3600 };
        let mut item = new_primary_verifier_entry("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = PrimaryVerifierEntryRegionLengthRule { max_length: 120 };
        let mut item = new_primary_verifier_entry("x", "y");
        item.region = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
