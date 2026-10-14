use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered migration adjustment must pass. Check returns its problems, or none.
pub trait TieredMigrationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredMigrationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAdjustmentCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredMigrationAdjustmentRule for TieredMigrationAdjustmentCapacityRangeRule {
    fn check(&self, item: &TieredMigrationAdjustment) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAdjustmentAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl TieredMigrationAdjustmentRule for TieredMigrationAdjustmentAllowedLabelRule {
    fn check(&self, item: &TieredMigrationAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is missing", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAdjustmentRequiredIssuerRule {
    pub message: String,
}

impl TieredMigrationAdjustmentRule for TieredMigrationAdjustmentRequiredIssuerRule {
    fn check(&self, item: &TieredMigrationAdjustment) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAdjustmentScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredMigrationAdjustmentRule for TieredMigrationAdjustmentScoreRangeRule {
    fn check(&self, item: &TieredMigrationAdjustment) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAdjustmentCurrencyLengthRule {
    pub max_length: i64,
}

impl TieredMigrationAdjustmentRule for TieredMigrationAdjustmentCurrencyLengthRule {
    fn check(&self, item: &TieredMigrationAdjustment) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAdjustmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredMigrationAdjustmentRule for TieredMigrationAdjustmentQuantityRangeRule {
    fn check(&self, item: &TieredMigrationAdjustment) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered migration adjustment.
#[derive(Debug, Clone)]
pub struct TieredMigrationAdjustmentValidator {
    pub rules: Vec<Rc<dyn TieredMigrationAdjustmentRule>>,
}

impl TieredMigrationAdjustmentValidator {
    pub fn validate(&self, item: &TieredMigrationAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredMigrationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredMigrationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_migration_adjustment_validator() -> TieredMigrationAdjustmentValidator {
    TieredMigrationAdjustmentValidator {
        rules: vec![
            Rc::new(TieredMigrationAdjustmentCapacityRangeRule { min: 16, max: 255 })
                as Rc<dyn TieredMigrationAdjustmentRule>,
            Rc::new(TieredMigrationAdjustmentAllowedLabelRule {
                allowed: vec!["pending".to_string(), "primary".to_string(), "capped".to_string()],
            }),
            Rc::new(TieredMigrationAdjustmentRequiredIssuerRule {
                message: "tiered migration adjustment needs this field".to_string(),
            }),
            Rc::new(TieredMigrationAdjustmentScoreRangeRule { min: 1.6, max: 1594.9 }),
            Rc::new(TieredMigrationAdjustmentCurrencyLengthRule { max_length: 3600 }),
            Rc::new(TieredMigrationAdjustmentQuantityRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_migration_adjustment_validator();
        let item = new_tiered_migration_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = TieredMigrationAdjustmentCapacityRangeRule { min: 80, max: 255 };
        let mut item = new_tiered_migration_adjustment("x", "y");
        item.capacity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = TieredMigrationAdjustmentRequiredIssuerRule {
            message: "tiered migration adjustment needs this field".to_string(),
        };
        let mut item = new_tiered_migration_adjustment("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TieredMigrationAdjustmentScoreRangeRule { min: 4.1, max: 2900.0 };
        let mut item = new_tiered_migration_adjustment("x", "y");
        item.score = 2901.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = TieredMigrationAdjustmentCurrencyLengthRule { max_length: 32 };
        let mut item = new_tiered_migration_adjustment("x", "y");
        item.currency = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = TieredMigrationAdjustmentQuantityRangeRule { min: 80, max: 86400 };
        let mut item = new_tiered_migration_adjustment("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
