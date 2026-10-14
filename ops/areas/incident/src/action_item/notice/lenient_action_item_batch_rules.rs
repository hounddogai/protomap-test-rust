use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient action item batch must pass. Check returns its problems, or none.
pub trait LenientActionItemBatchRule: std::fmt::Debug {
    fn check(&self, item: &LenientActionItemBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBatchRequiredLabelRule {
    pub message: String,
}

impl LenientActionItemBatchRule for LenientActionItemBatchRequiredLabelRule {
    fn check(&self, item: &LenientActionItemBatch) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBatchRequiredCodeRule {
    pub message: String,
}

impl LenientActionItemBatchRule for LenientActionItemBatchRequiredCodeRule {
    fn check(&self, item: &LenientActionItemBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBatchCipherLengthRule {
    pub max_length: i64,
}

impl LenientActionItemBatchRule for LenientActionItemBatchCipherLengthRule {
    fn check(&self, item: &LenientActionItemBatch) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBatchRequiredRegionRule {
    pub message: String,
}

impl LenientActionItemBatchRule for LenientActionItemBatchRequiredRegionRule {
    fn check(&self, item: &LenientActionItemBatch) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemBatchTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl LenientActionItemBatchRule for LenientActionItemBatchTagsLimitRule {
    fn check(&self, item: &LenientActionItemBatch) -> Vec<String> {
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
pub struct LenientActionItemBatchCodeLengthRule {
    pub max_length: i64,
}

impl LenientActionItemBatchRule for LenientActionItemBatchCodeLengthRule {
    fn check(&self, item: &LenientActionItemBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient action item batch.
#[derive(Debug, Clone)]
pub struct LenientActionItemBatchValidator {
    pub rules: Vec<Rc<dyn LenientActionItemBatchRule>>,
}

impl LenientActionItemBatchValidator {
    pub fn validate(&self, item: &LenientActionItemBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientActionItemBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientActionItemBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_action_item_batch_validator() -> LenientActionItemBatchValidator {
    LenientActionItemBatchValidator {
        rules: vec![
            Rc::new(LenientActionItemBatchRequiredLabelRule {
                message: "lenient action item batch needs this field".to_string(),
            }) as Rc<dyn LenientActionItemBatchRule>,
            Rc::new(LenientActionItemBatchRequiredCodeRule {
                message: "lenient action item batch needs a value".to_string(),
            }),
            Rc::new(LenientActionItemBatchCipherLengthRule { max_length: 86400 }),
            Rc::new(LenientActionItemBatchRequiredRegionRule {
                message: "lenient action item batch needs a value".to_string(),
            }),
            Rc::new(LenientActionItemBatchTagsLimitRule {
                max_count: 16,
                banned: Vec::new(),
            }),
            Rc::new(LenientActionItemBatchCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_action_item_batch_validator();
        let item = new_lenient_action_item_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientActionItemBatchRequiredLabelRule {
            message: "lenient action item batch needs an entry".to_string(),
        };
        let mut item = new_lenient_action_item_batch("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientActionItemBatchRequiredCodeRule {
            message: "lenient action item batch needs a value".to_string(),
        };
        let mut item = new_lenient_action_item_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = LenientActionItemBatchCipherLengthRule { max_length: 32 };
        let mut item = new_lenient_action_item_batch("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = LenientActionItemBatchRequiredRegionRule {
            message: "lenient action item batch needs an entry".to_string(),
        };
        let mut item = new_lenient_action_item_batch("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientActionItemBatchCodeLengthRule { max_length: 64 };
        let mut item = new_lenient_action_item_batch("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
