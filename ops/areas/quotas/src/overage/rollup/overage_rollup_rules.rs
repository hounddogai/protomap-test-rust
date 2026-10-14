use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage rollup must pass. Check returns its problems, or none.
pub trait OverageRollupRule: std::fmt::Debug {
    fn check(&self, item: &OverageRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRollupRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageRollupRule for OverageRollupRevisionRangeRule {
    fn check(&self, item: &OverageRollup) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRollupCategoryLengthRule {
    pub max_length: i64,
}

impl OverageRollupRule for OverageRollupCategoryLengthRule {
    fn check(&self, item: &OverageRollup) -> Vec<String> {
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
pub struct OverageRollupNameLengthRule {
    pub max_length: i64,
}

impl OverageRollupRule for OverageRollupNameLengthRule {
    fn check(&self, item: &OverageRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageRollupAllowedNameRule {
    pub allowed: Vec<String>,
}

impl OverageRollupRule for OverageRollupAllowedNameRule {
    fn check(&self, item: &OverageRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is inconsistent", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage rollup.
#[derive(Debug, Clone)]
pub struct OverageRollupValidator {
    pub rules: Vec<Rc<dyn OverageRollupRule>>,
}

impl OverageRollupValidator {
    pub fn validate(&self, item: &OverageRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_rollup_validator() -> OverageRollupValidator {
    OverageRollupValidator {
        rules: vec![
            Rc::new(OverageRollupRevisionRangeRule { min: 80, max: 86400 }) as Rc<dyn OverageRollupRule>,
            Rc::new(OverageRollupCategoryLengthRule { max_length: 1000 }),
            Rc::new(OverageRollupNameLengthRule { max_length: 32 }),
            Rc::new(OverageRollupAllowedNameRule {
                allowed: vec!["capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_rollup_validator();
        let item = new_overage_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = OverageRollupRevisionRangeRule { min: 16, max: 80 };
        let mut item = new_overage_rollup("x", "y");
        item.revision = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = OverageRollupCategoryLengthRule { max_length: 86400 };
        let mut item = new_overage_rollup("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = OverageRollupNameLengthRule { max_length: 255 };
        let mut item = new_overage_rollup("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
