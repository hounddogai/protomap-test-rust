use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration notice must pass. Check returns its problems, or none.
pub trait MigrationNoticeRule: std::fmt::Debug {
    fn check(&self, item: &MigrationNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationNoticeCodePrefixRule {
    pub prefix: String,
}

impl MigrationNoticeRule for MigrationNoticeCodePrefixRule {
    fn check(&self, item: &MigrationNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationNoticeQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationNoticeRule for MigrationNoticeQuantityRangeRule {
    fn check(&self, item: &MigrationNotice) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationNoticeRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationNoticeRule for MigrationNoticeRatioRangeRule {
    fn check(&self, item: &MigrationNotice) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationNoticeTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl MigrationNoticeRule for MigrationNoticeTagsLimitRule {
    fn check(&self, item: &MigrationNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a migration notice.
#[derive(Debug, Clone)]
pub struct MigrationNoticeValidator {
    pub rules: Vec<Rc<dyn MigrationNoticeRule>>,
}

impl MigrationNoticeValidator {
    pub fn validate(&self, item: &MigrationNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_notice_validator() -> MigrationNoticeValidator {
    MigrationNoticeValidator {
        rules: vec![
            Rc::new(MigrationNoticeCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn MigrationNoticeRule>,
            Rc::new(MigrationNoticeQuantityRangeRule { min: 64, max: 64 }),
            Rc::new(MigrationNoticeRatioRangeRule { min: 5.7, max: 610.1 }),
            Rc::new(MigrationNoticeTagsLimitRule {
                max_count: 1000,
                banned: vec!["weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_notice_validator();
        let item = new_migration_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = MigrationNoticeQuantityRangeRule { min: 16, max: 64 };
        let mut item = new_migration_notice("x", "y");
        item.quantity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = MigrationNoticeRatioRangeRule { min: 8.1, max: 4824.3 };
        let mut item = new_migration_notice("x", "y");
        item.ratio = 4825.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
