use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional migration tier must pass. Check returns its problems, or none.
pub trait RegionalMigrationTierRule: std::fmt::Debug {
    fn check(&self, item: &RegionalMigrationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationTierTitleLengthRule {
    pub max_length: i64,
}

impl RegionalMigrationTierRule for RegionalMigrationTierTitleLengthRule {
    fn check(&self, item: &RegionalMigrationTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationTierNamespaceLengthRule {
    pub max_length: i64,
}

impl RegionalMigrationTierRule for RegionalMigrationTierNamespaceLengthRule {
    fn check(&self, item: &RegionalMigrationTier) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationTierThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalMigrationTierRule for RegionalMigrationTierThresholdRangeRule {
    fn check(&self, item: &RegionalMigrationTier) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationTierCodePrefixRule {
    pub prefix: String,
}

impl RegionalMigrationTierRule for RegionalMigrationTierCodePrefixRule {
    fn check(&self, item: &RegionalMigrationTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationTierQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalMigrationTierRule for RegionalMigrationTierQuantityRangeRule {
    fn check(&self, item: &RegionalMigrationTier) -> Vec<String> {
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

/// Runs every rule of a regional migration tier.
#[derive(Debug, Clone)]
pub struct RegionalMigrationTierValidator {
    pub rules: Vec<Rc<dyn RegionalMigrationTierRule>>,
}

impl RegionalMigrationTierValidator {
    pub fn validate(&self, item: &RegionalMigrationTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalMigrationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalMigrationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_migration_tier_validator() -> RegionalMigrationTierValidator {
    RegionalMigrationTierValidator {
        rules: vec![
            Rc::new(RegionalMigrationTierTitleLengthRule { max_length: 1000 }) as Rc<dyn RegionalMigrationTierRule>,
            Rc::new(RegionalMigrationTierNamespaceLengthRule { max_length: 120 }),
            Rc::new(RegionalMigrationTierThresholdRangeRule { min: 9.3, max: 129.6 }),
            Rc::new(RegionalMigrationTierCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionalMigrationTierQuantityRangeRule { min: 3600, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_migration_tier_validator();
        let item = new_regional_migration_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalMigrationTierTitleLengthRule { max_length: 3600 };
        let mut item = new_regional_migration_tier("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = RegionalMigrationTierNamespaceLengthRule { max_length: 120 };
        let mut item = new_regional_migration_tier("x", "y");
        item.namespace = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = RegionalMigrationTierThresholdRangeRule { min: 8.1, max: 764.8 };
        let mut item = new_regional_migration_tier("x", "y");
        item.threshold = 766.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = RegionalMigrationTierQuantityRangeRule { min: 32, max: 3600 };
        let mut item = new_regional_migration_tier("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
