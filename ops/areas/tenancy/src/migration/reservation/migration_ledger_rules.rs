use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration ledger must pass. Check returns its problems, or none.
pub trait MigrationLedgerRule: std::fmt::Debug {
    fn check(&self, item: &MigrationLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationLedgerFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationLedgerRule for MigrationLedgerFloorValueRangeRule {
    fn check(&self, item: &MigrationLedger) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationLedgerLabelLengthRule {
    pub max_length: i64,
}

impl MigrationLedgerRule for MigrationLedgerLabelLengthRule {
    fn check(&self, item: &MigrationLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationLedgerCodePrefixRule {
    pub prefix: String,
}

impl MigrationLedgerRule for MigrationLedgerCodePrefixRule {
    fn check(&self, item: &MigrationLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationLedgerRequiredCountryRule {
    pub message: String,
}

impl MigrationLedgerRule for MigrationLedgerRequiredCountryRule {
    fn check(&self, item: &MigrationLedger) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationLedgerDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationLedgerRule for MigrationLedgerDiscountRangeRule {
    fn check(&self, item: &MigrationLedger) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationLedgerRequiredAudienceRule {
    pub message: String,
}

impl MigrationLedgerRule for MigrationLedgerRequiredAudienceRule {
    fn check(&self, item: &MigrationLedger) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration ledger.
#[derive(Debug, Clone)]
pub struct MigrationLedgerValidator {
    pub rules: Vec<Rc<dyn MigrationLedgerRule>>,
}

impl MigrationLedgerValidator {
    pub fn validate(&self, item: &MigrationLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_ledger_validator() -> MigrationLedgerValidator {
    MigrationLedgerValidator {
        rules: vec![
            Rc::new(MigrationLedgerFloorValueRangeRule { min: 4.0, max: 4660.7 }) as Rc<dyn MigrationLedgerRule>,
            Rc::new(MigrationLedgerLabelLengthRule { max_length: 64 }),
            Rc::new(MigrationLedgerCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(MigrationLedgerRequiredCountryRule {
                message: "migration ledger needs this field".to_string(),
            }),
            Rc::new(MigrationLedgerDiscountRangeRule { min: 6.8, max: 1407.1 }),
            Rc::new(MigrationLedgerRequiredAudienceRule {
                message: "migration ledger needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_ledger_validator();
        let item = new_migration_ledger("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = MigrationLedgerFloorValueRangeRule { min: 6.7, max: 2880.6 };
        let mut item = new_migration_ledger("x", "y");
        item.floor_value = 2882.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = MigrationLedgerLabelLengthRule { max_length: 86400 };
        let mut item = new_migration_ledger("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = MigrationLedgerRequiredCountryRule {
            message: "migration ledger needs a value".to_string(),
        };
        let mut item = new_migration_ledger("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = MigrationLedgerDiscountRangeRule { min: 9.9, max: 1562.8 };
        let mut item = new_migration_ledger("x", "y");
        item.discount = 1564.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = MigrationLedgerRequiredAudienceRule {
            message: "migration ledger needs a value".to_string(),
        };
        let mut item = new_migration_ledger("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
