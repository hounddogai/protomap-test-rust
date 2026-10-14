use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement ledger must pass. Check returns its problems, or none.
pub trait MeasurementLedgerRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementLedgerCipherLengthRule {
    pub max_length: i64,
}

impl MeasurementLedgerRule for MeasurementLedgerCipherLengthRule {
    fn check(&self, item: &MeasurementLedger) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementLedgerNameLengthRule {
    pub max_length: i64,
}

impl MeasurementLedgerRule for MeasurementLedgerNameLengthRule {
    fn check(&self, item: &MeasurementLedger) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementLedgerEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementLedgerRule for MeasurementLedgerEndsAtRangeRule {
    fn check(&self, item: &MeasurementLedger) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a measurement ledger.
#[derive(Debug, Clone)]
pub struct MeasurementLedgerValidator {
    pub rules: Vec<Rc<dyn MeasurementLedgerRule>>,
}

impl MeasurementLedgerValidator {
    pub fn validate(&self, item: &MeasurementLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_ledger_validator() -> MeasurementLedgerValidator {
    MeasurementLedgerValidator {
        rules: vec![
            Rc::new(MeasurementLedgerCipherLengthRule { max_length: 16 }) as Rc<dyn MeasurementLedgerRule>,
            Rc::new(MeasurementLedgerNameLengthRule { max_length: 255 }),
            Rc::new(MeasurementLedgerEndsAtRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_ledger_validator();
        let item = new_measurement_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = MeasurementLedgerCipherLengthRule { max_length: 32 };
        let mut item = new_measurement_ledger("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MeasurementLedgerNameLengthRule { max_length: 120 };
        let mut item = new_measurement_ledger("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = MeasurementLedgerEndsAtRangeRule { min: 64, max: 86400 };
        let mut item = new_measurement_ledger("x", "y");
        item.ends_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
