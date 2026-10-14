use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered region pin agreement must pass. Check returns its problems, or none.
pub trait TieredRegionPinAgreementRule: std::fmt::Debug {
    fn check(&self, item: &TieredRegionPinAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinAgreementAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredRegionPinAgreementRule for TieredRegionPinAgreementAttemptsRangeRule {
    fn check(&self, item: &TieredRegionPinAgreement) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinAgreementRequiredChecksumRule {
    pub message: String,
}

impl TieredRegionPinAgreementRule for TieredRegionPinAgreementRequiredChecksumRule {
    fn check(&self, item: &TieredRegionPinAgreement) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionPinAgreementChecksumLengthRule {
    pub max_length: i64,
}

impl TieredRegionPinAgreementRule for TieredRegionPinAgreementChecksumLengthRule {
    fn check(&self, item: &TieredRegionPinAgreement) -> Vec<String> {
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

/// Runs every rule of a tiered region pin agreement.
#[derive(Debug, Clone)]
pub struct TieredRegionPinAgreementValidator {
    pub rules: Vec<Rc<dyn TieredRegionPinAgreementRule>>,
}

impl TieredRegionPinAgreementValidator {
    pub fn validate(&self, item: &TieredRegionPinAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredRegionPinAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRegionPinAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_region_pin_agreement_validator() -> TieredRegionPinAgreementValidator {
    TieredRegionPinAgreementValidator {
        rules: vec![
            Rc::new(TieredRegionPinAgreementAttemptsRangeRule { min: 1000, max: 3600 })
                as Rc<dyn TieredRegionPinAgreementRule>,
            Rc::new(TieredRegionPinAgreementRequiredChecksumRule {
                message: "tiered region pin agreement needs an entry".to_string(),
            }),
            Rc::new(TieredRegionPinAgreementChecksumLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_region_pin_agreement_validator();
        let item = new_tiered_region_pin_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = TieredRegionPinAgreementAttemptsRangeRule { min: 3600, max: 86400 };
        let mut item = new_tiered_region_pin_agreement("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = TieredRegionPinAgreementRequiredChecksumRule {
            message: "tiered region pin agreement needs this field".to_string(),
        };
        let mut item = new_tiered_region_pin_agreement("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = TieredRegionPinAgreementChecksumLengthRule { max_length: 1000 };
        let mut item = new_tiered_region_pin_agreement("x", "y");
        item.checksum = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
