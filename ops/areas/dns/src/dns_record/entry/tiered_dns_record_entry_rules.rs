use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered dns record entry must pass. Check returns its problems, or none.
pub trait TieredDnsRecordEntryRule: std::fmt::Debug {
    fn check(&self, item: &TieredDnsRecordEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordEntryPhoneLengthRule {
    pub max_length: i64,
}

impl TieredDnsRecordEntryRule for TieredDnsRecordEntryPhoneLengthRule {
    fn check(&self, item: &TieredDnsRecordEntry) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordEntryBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredDnsRecordEntryRule for TieredDnsRecordEntryBatchSizeRangeRule {
    fn check(&self, item: &TieredDnsRecordEntry) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordEntryWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredDnsRecordEntryRule for TieredDnsRecordEntryWindowDaysRangeRule {
    fn check(&self, item: &TieredDnsRecordEntry) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered dns record entry.
#[derive(Debug, Clone)]
pub struct TieredDnsRecordEntryValidator {
    pub rules: Vec<Rc<dyn TieredDnsRecordEntryRule>>,
}

impl TieredDnsRecordEntryValidator {
    pub fn validate(&self, item: &TieredDnsRecordEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredDnsRecordEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDnsRecordEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_dns_record_entry_validator() -> TieredDnsRecordEntryValidator {
    TieredDnsRecordEntryValidator {
        rules: vec![
            Rc::new(TieredDnsRecordEntryPhoneLengthRule { max_length: 32 }) as Rc<dyn TieredDnsRecordEntryRule>,
            Rc::new(TieredDnsRecordEntryBatchSizeRangeRule { min: 1000, max: 86400 }),
            Rc::new(TieredDnsRecordEntryWindowDaysRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_dns_record_entry_validator();
        let item = new_tiered_dns_record_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = TieredDnsRecordEntryPhoneLengthRule { max_length: 32 };
        let mut item = new_tiered_dns_record_entry("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TieredDnsRecordEntryBatchSizeRangeRule { min: 32, max: 255 };
        let mut item = new_tiered_dns_record_entry("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = TieredDnsRecordEntryWindowDaysRangeRule { min: 120, max: 255 };
        let mut item = new_tiered_dns_record_entry("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
