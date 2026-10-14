use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy migration adjustment must pass. Check returns its problems, or none.
pub trait LegacyMigrationAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyMigrationAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl LegacyMigrationAdjustmentRule for LegacyMigrationAdjustmentCodeLengthRule {
    fn check(&self, item: &LegacyMigrationAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAdjustmentScopeLengthRule {
    pub max_length: i64,
}

impl LegacyMigrationAdjustmentRule for LegacyMigrationAdjustmentScopeLengthRule {
    fn check(&self, item: &LegacyMigrationAdjustment) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAdjustmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyMigrationAdjustmentRule for LegacyMigrationAdjustmentCostRangeRule {
    fn check(&self, item: &LegacyMigrationAdjustment) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAdjustmentRequiredScopeRule {
    pub message: String,
}

impl LegacyMigrationAdjustmentRule for LegacyMigrationAdjustmentRequiredScopeRule {
    fn check(&self, item: &LegacyMigrationAdjustment) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAdjustmentAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl LegacyMigrationAdjustmentRule for LegacyMigrationAdjustmentAllowedOwnerIdRule {
    fn check(&self, item: &LegacyMigrationAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is unknown", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl LegacyMigrationAdjustmentRule for LegacyMigrationAdjustmentTitleLengthRule {
    fn check(&self, item: &LegacyMigrationAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy migration adjustment.
#[derive(Debug, Clone)]
pub struct LegacyMigrationAdjustmentValidator {
    pub rules: Vec<Rc<dyn LegacyMigrationAdjustmentRule>>,
}

impl LegacyMigrationAdjustmentValidator {
    pub fn validate(&self, item: &LegacyMigrationAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyMigrationAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyMigrationAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_migration_adjustment_validator() -> LegacyMigrationAdjustmentValidator {
    LegacyMigrationAdjustmentValidator {
        rules: vec![
            Rc::new(LegacyMigrationAdjustmentCodeLengthRule { max_length: 120 })
                as Rc<dyn LegacyMigrationAdjustmentRule>,
            Rc::new(LegacyMigrationAdjustmentScopeLengthRule { max_length: 16 }),
            Rc::new(LegacyMigrationAdjustmentCostRangeRule { min: 6.6, max: 2072.3 }),
            Rc::new(LegacyMigrationAdjustmentRequiredScopeRule {
                message: "legacy migration adjustment needs this field".to_string(),
            }),
            Rc::new(LegacyMigrationAdjustmentAllowedOwnerIdRule {
                allowed: vec!["legacy".to_string(), "rolling".to_string()],
            }),
            Rc::new(LegacyMigrationAdjustmentTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_migration_adjustment_validator();
        let item = new_legacy_migration_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyMigrationAdjustmentCodeLengthRule { max_length: 3600 };
        let mut item = new_legacy_migration_adjustment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = LegacyMigrationAdjustmentScopeLengthRule { max_length: 1000 };
        let mut item = new_legacy_migration_adjustment("x", "y");
        item.scope = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = LegacyMigrationAdjustmentCostRangeRule { min: 7.8, max: 3182.9 };
        let mut item = new_legacy_migration_adjustment("x", "y");
        item.cost = 3184.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = LegacyMigrationAdjustmentRequiredScopeRule {
            message: "legacy migration adjustment needs an entry".to_string(),
        };
        let mut item = new_legacy_migration_adjustment("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyMigrationAdjustmentTitleLengthRule { max_length: 3600 };
        let mut item = new_legacy_migration_adjustment("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
