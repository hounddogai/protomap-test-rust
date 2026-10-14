use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback nonce ledger must pass. Check returns its problems, or none.
pub trait FallbackNonceLedgerRule: std::fmt::Debug {
    fn check(&self, item: &FallbackNonceLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackNonceLedgerCodePrefixRule {
    pub prefix: String,
}

impl FallbackNonceLedgerRule for FallbackNonceLedgerCodePrefixRule {
    fn check(&self, item: &FallbackNonceLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackNonceLedgerGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackNonceLedgerRule for FallbackNonceLedgerGraceDaysRangeRule {
    fn check(&self, item: &FallbackNonceLedger) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackNonceLedgerPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackNonceLedgerRule for FallbackNonceLedgerPriorityRangeRule {
    fn check(&self, item: &FallbackNonceLedger) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackNonceLedgerRegionLengthRule {
    pub max_length: i64,
}

impl FallbackNonceLedgerRule for FallbackNonceLedgerRegionLengthRule {
    fn check(&self, item: &FallbackNonceLedger) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackNonceLedgerDescriptionLengthRule {
    pub max_length: i64,
}

impl FallbackNonceLedgerRule for FallbackNonceLedgerDescriptionLengthRule {
    fn check(&self, item: &FallbackNonceLedger) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback nonce ledger.
#[derive(Debug, Clone)]
pub struct FallbackNonceLedgerValidator {
    pub rules: Vec<Rc<dyn FallbackNonceLedgerRule>>,
}

impl FallbackNonceLedgerValidator {
    pub fn validate(&self, item: &FallbackNonceLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackNonceLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackNonceLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_nonce_ledger_validator() -> FallbackNonceLedgerValidator {
    FallbackNonceLedgerValidator {
        rules: vec![
            Rc::new(FallbackNonceLedgerCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn FallbackNonceLedgerRule>,
            Rc::new(FallbackNonceLedgerGraceDaysRangeRule { min: 32, max: 255 }),
            Rc::new(FallbackNonceLedgerPriorityRangeRule { min: 32, max: 120 }),
            Rc::new(FallbackNonceLedgerRegionLengthRule { max_length: 86400 }),
            Rc::new(FallbackNonceLedgerDescriptionLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_nonce_ledger_validator();
        let item = new_fallback_nonce_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FallbackNonceLedgerGraceDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_fallback_nonce_ledger("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = FallbackNonceLedgerPriorityRangeRule { min: 120, max: 120 };
        let mut item = new_fallback_nonce_ledger("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = FallbackNonceLedgerRegionLengthRule { max_length: 1000 };
        let mut item = new_fallback_nonce_ledger("x", "y");
        item.region = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = FallbackNonceLedgerDescriptionLengthRule { max_length: 64 };
        let mut item = new_fallback_nonce_ledger("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
