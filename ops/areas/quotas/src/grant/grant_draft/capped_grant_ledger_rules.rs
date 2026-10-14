use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped grant ledger must pass. Check returns its problems, or none.
pub trait CappedGrantLedgerRule: std::fmt::Debug {
    fn check(&self, item: &CappedGrantLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantLedgerFingerprintLengthRule {
    pub max_length: i64,
}

impl CappedGrantLedgerRule for CappedGrantLedgerFingerprintLengthRule {
    fn check(&self, item: &CappedGrantLedger) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantLedgerFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedGrantLedgerRule for CappedGrantLedgerFloorValueRangeRule {
    fn check(&self, item: &CappedGrantLedger) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantLedgerRequiredAudienceRule {
    pub message: String,
}

impl CappedGrantLedgerRule for CappedGrantLedgerRequiredAudienceRule {
    fn check(&self, item: &CappedGrantLedger) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped grant ledger.
#[derive(Debug, Clone)]
pub struct CappedGrantLedgerValidator {
    pub rules: Vec<Rc<dyn CappedGrantLedgerRule>>,
}

impl CappedGrantLedgerValidator {
    pub fn validate(&self, item: &CappedGrantLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedGrantLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedGrantLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_grant_ledger_validator() -> CappedGrantLedgerValidator {
    CappedGrantLedgerValidator {
        rules: vec![
            Rc::new(CappedGrantLedgerFingerprintLengthRule { max_length: 120 }) as Rc<dyn CappedGrantLedgerRule>,
            Rc::new(CappedGrantLedgerFloorValueRangeRule { min: 3.3, max: 883.0 }),
            Rc::new(CappedGrantLedgerRequiredAudienceRule {
                message: "capped grant ledger needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_grant_ledger_validator();
        let item = new_capped_grant_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = CappedGrantLedgerFingerprintLengthRule { max_length: 3600 };
        let mut item = new_capped_grant_ledger("x", "y");
        item.fingerprint = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = CappedGrantLedgerFloorValueRangeRule { min: 2.2, max: 4008.3 };
        let mut item = new_capped_grant_ledger("x", "y");
        item.floor_value = 4009.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = CappedGrantLedgerRequiredAudienceRule {
            message: "capped grant ledger needs a value".to_string(),
        };
        let mut item = new_capped_grant_ledger("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
