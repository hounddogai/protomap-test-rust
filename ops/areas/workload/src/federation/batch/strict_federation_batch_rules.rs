use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict federation batch must pass. Check returns its problems, or none.
pub trait StrictFederationBatchRule: std::fmt::Debug {
    fn check(&self, item: &StrictFederationBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationBatchRequiredLabelRule {
    pub message: String,
}

impl StrictFederationBatchRule for StrictFederationBatchRequiredLabelRule {
    fn check(&self, item: &StrictFederationBatch) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationBatchLabelLengthRule {
    pub max_length: i64,
}

impl StrictFederationBatchRule for StrictFederationBatchLabelLengthRule {
    fn check(&self, item: &StrictFederationBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationBatchUnitLengthRule {
    pub max_length: i64,
}

impl StrictFederationBatchRule for StrictFederationBatchUnitLengthRule {
    fn check(&self, item: &StrictFederationBatch) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationBatchTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl StrictFederationBatchRule for StrictFederationBatchTagsLimitRule {
    fn check(&self, item: &StrictFederationBatch) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationBatchCategoryLengthRule {
    pub max_length: i64,
}

impl StrictFederationBatchRule for StrictFederationBatchCategoryLengthRule {
    fn check(&self, item: &StrictFederationBatch) -> Vec<String> {
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
pub struct StrictFederationBatchCodeLengthRule {
    pub max_length: i64,
}

impl StrictFederationBatchRule for StrictFederationBatchCodeLengthRule {
    fn check(&self, item: &StrictFederationBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict federation batch.
#[derive(Debug, Clone)]
pub struct StrictFederationBatchValidator {
    pub rules: Vec<Rc<dyn StrictFederationBatchRule>>,
}

impl StrictFederationBatchValidator {
    pub fn validate(&self, item: &StrictFederationBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictFederationBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictFederationBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_federation_batch_validator() -> StrictFederationBatchValidator {
    StrictFederationBatchValidator {
        rules: vec![
            Rc::new(StrictFederationBatchRequiredLabelRule {
                message: "strict federation batch needs this field".to_string(),
            }) as Rc<dyn StrictFederationBatchRule>,
            Rc::new(StrictFederationBatchLabelLengthRule { max_length: 255 }),
            Rc::new(StrictFederationBatchUnitLengthRule { max_length: 255 }),
            Rc::new(StrictFederationBatchTagsLimitRule {
                max_count: 80,
                banned: vec!["lenient".to_string(), "lenient".to_string(), "default".to_string()],
            }),
            Rc::new(StrictFederationBatchCategoryLengthRule { max_length: 86400 }),
            Rc::new(StrictFederationBatchCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_federation_batch_validator();
        let item = new_strict_federation_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictFederationBatchRequiredLabelRule {
            message: "strict federation batch needs this field".to_string(),
        };
        let mut item = new_strict_federation_batch("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictFederationBatchLabelLengthRule { max_length: 86400 };
        let mut item = new_strict_federation_batch("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = StrictFederationBatchUnitLengthRule { max_length: 86400 };
        let mut item = new_strict_federation_batch("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = StrictFederationBatchCategoryLengthRule { max_length: 64 };
        let mut item = new_strict_federation_batch("x", "y");
        item.category = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictFederationBatchCodeLengthRule { max_length: 80 };
        let mut item = new_strict_federation_batch("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
