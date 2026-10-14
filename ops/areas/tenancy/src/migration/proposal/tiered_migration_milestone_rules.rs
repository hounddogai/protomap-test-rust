use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered migration milestone must pass. Check returns its problems, or none.
pub trait TieredMigrationMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &TieredMigrationMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMilestoneRequiredCodeRule {
    pub message: String,
}

impl TieredMigrationMilestoneRule for TieredMigrationMilestoneRequiredCodeRule {
    fn check(&self, item: &TieredMigrationMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMilestoneScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredMigrationMilestoneRule for TieredMigrationMilestoneScoreRangeRule {
    fn check(&self, item: &TieredMigrationMilestone) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMilestoneRequiredNameRule {
    pub message: String,
}

impl TieredMigrationMilestoneRule for TieredMigrationMilestoneRequiredNameRule {
    fn check(&self, item: &TieredMigrationMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMilestoneRegionLengthRule {
    pub max_length: i64,
}

impl TieredMigrationMilestoneRule for TieredMigrationMilestoneRegionLengthRule {
    fn check(&self, item: &TieredMigrationMilestone) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered migration milestone.
#[derive(Debug, Clone)]
pub struct TieredMigrationMilestoneValidator {
    pub rules: Vec<Rc<dyn TieredMigrationMilestoneRule>>,
}

impl TieredMigrationMilestoneValidator {
    pub fn validate(&self, item: &TieredMigrationMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredMigrationMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredMigrationMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_migration_milestone_validator() -> TieredMigrationMilestoneValidator {
    TieredMigrationMilestoneValidator {
        rules: vec![
            Rc::new(TieredMigrationMilestoneRequiredCodeRule {
                message: "tiered migration milestone needs a value".to_string(),
            }) as Rc<dyn TieredMigrationMilestoneRule>,
            Rc::new(TieredMigrationMilestoneScoreRangeRule { min: 5.5, max: 3862.3 }),
            Rc::new(TieredMigrationMilestoneRequiredNameRule {
                message: "tiered migration milestone needs an entry".to_string(),
            }),
            Rc::new(TieredMigrationMilestoneRegionLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_migration_milestone_validator();
        let item = new_tiered_migration_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredMigrationMilestoneRequiredCodeRule {
            message: "tiered migration milestone needs this field".to_string(),
        };
        let mut item = new_tiered_migration_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = TieredMigrationMilestoneScoreRangeRule { min: 3.4, max: 4754.0 };
        let mut item = new_tiered_migration_milestone("x", "y");
        item.score = 4755.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredMigrationMilestoneRequiredNameRule {
            message: "tiered migration milestone needs this field".to_string(),
        };
        let mut item = new_tiered_migration_milestone("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = TieredMigrationMilestoneRegionLengthRule { max_length: 255 };
        let mut item = new_tiered_migration_milestone("x", "y");
        item.region = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
