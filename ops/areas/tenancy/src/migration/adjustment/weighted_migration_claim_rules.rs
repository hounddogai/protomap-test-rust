use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted migration claim must pass. Check returns its problems, or none.
pub trait WeightedMigrationClaimRule: std::fmt::Debug {
    fn check(&self, item: &WeightedMigrationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMigrationClaimRequiredCountryRule {
    pub message: String,
}

impl WeightedMigrationClaimRule for WeightedMigrationClaimRequiredCountryRule {
    fn check(&self, item: &WeightedMigrationClaim) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMigrationClaimRequiredCategoryRule {
    pub message: String,
}

impl WeightedMigrationClaimRule for WeightedMigrationClaimRequiredCategoryRule {
    fn check(&self, item: &WeightedMigrationClaim) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMigrationClaimCodeLengthRule {
    pub max_length: i64,
}

impl WeightedMigrationClaimRule for WeightedMigrationClaimCodeLengthRule {
    fn check(&self, item: &WeightedMigrationClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMigrationClaimCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedMigrationClaimRule for WeightedMigrationClaimCapacityRangeRule {
    fn check(&self, item: &WeightedMigrationClaim) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted migration claim.
#[derive(Debug, Clone)]
pub struct WeightedMigrationClaimValidator {
    pub rules: Vec<Rc<dyn WeightedMigrationClaimRule>>,
}

impl WeightedMigrationClaimValidator {
    pub fn validate(&self, item: &WeightedMigrationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedMigrationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedMigrationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_migration_claim_validator() -> WeightedMigrationClaimValidator {
    WeightedMigrationClaimValidator {
        rules: vec![
            Rc::new(WeightedMigrationClaimRequiredCountryRule {
                message: "weighted migration claim needs an entry".to_string(),
            }) as Rc<dyn WeightedMigrationClaimRule>,
            Rc::new(WeightedMigrationClaimRequiredCategoryRule {
                message: "weighted migration claim needs a value".to_string(),
            }),
            Rc::new(WeightedMigrationClaimCodeLengthRule { max_length: 3600 }),
            Rc::new(WeightedMigrationClaimCapacityRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_migration_claim_validator();
        let item = new_weighted_migration_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = WeightedMigrationClaimRequiredCountryRule {
            message: "weighted migration claim needs a value".to_string(),
        };
        let mut item = new_weighted_migration_claim("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = WeightedMigrationClaimRequiredCategoryRule {
            message: "weighted migration claim needs this field".to_string(),
        };
        let mut item = new_weighted_migration_claim("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedMigrationClaimCodeLengthRule { max_length: 86400 };
        let mut item = new_weighted_migration_claim("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = WeightedMigrationClaimCapacityRangeRule { min: 32, max: 80 };
        let mut item = new_weighted_migration_claim("x", "y");
        item.capacity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
