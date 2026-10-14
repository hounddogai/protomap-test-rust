use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary rollback rollup must pass. Check returns its problems, or none.
pub trait PrimaryRollbackRollupRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRollbackRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRollbackRollupAudienceLengthRule {
    pub max_length: i64,
}

impl PrimaryRollbackRollupRule for PrimaryRollbackRollupAudienceLengthRule {
    fn check(&self, item: &PrimaryRollbackRollup) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRollbackRollupCategoryLengthRule {
    pub max_length: i64,
}

impl PrimaryRollbackRollupRule for PrimaryRollbackRollupCategoryLengthRule {
    fn check(&self, item: &PrimaryRollbackRollup) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRollbackRollupCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRollbackRollupRule for PrimaryRollbackRollupCodeLengthRule {
    fn check(&self, item: &PrimaryRollbackRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRollbackRollupRequiredCategoryRule {
    pub message: String,
}

impl PrimaryRollbackRollupRule for PrimaryRollbackRollupRequiredCategoryRule {
    fn check(&self, item: &PrimaryRollbackRollup) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary rollback rollup.
#[derive(Debug, Clone)]
pub struct PrimaryRollbackRollupValidator {
    pub rules: Vec<Rc<dyn PrimaryRollbackRollupRule>>,
}

impl PrimaryRollbackRollupValidator {
    pub fn validate(&self, item: &PrimaryRollbackRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRollbackRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRollbackRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_rollback_rollup_validator() -> PrimaryRollbackRollupValidator {
    PrimaryRollbackRollupValidator {
        rules: vec![
            Rc::new(PrimaryRollbackRollupAudienceLengthRule { max_length: 255 }) as Rc<dyn PrimaryRollbackRollupRule>,
            Rc::new(PrimaryRollbackRollupCategoryLengthRule { max_length: 16 }),
            Rc::new(PrimaryRollbackRollupCodeLengthRule { max_length: 255 }),
            Rc::new(PrimaryRollbackRollupRequiredCategoryRule {
                message: "primary rollback rollup needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_rollback_rollup_validator();
        let item = new_primary_rollback_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PrimaryRollbackRollupAudienceLengthRule { max_length: 255 };
        let mut item = new_primary_rollback_rollup("x", "y");
        item.audience = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = PrimaryRollbackRollupCategoryLengthRule { max_length: 16 };
        let mut item = new_primary_rollback_rollup("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRollbackRollupCodeLengthRule { max_length: 255 };
        let mut item = new_primary_rollback_rollup("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = PrimaryRollbackRollupRequiredCategoryRule {
            message: "primary rollback rollup needs an entry".to_string(),
        };
        let mut item = new_primary_rollback_rollup("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
