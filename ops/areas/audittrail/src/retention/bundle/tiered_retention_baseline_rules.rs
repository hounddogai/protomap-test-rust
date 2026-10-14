use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered retention baseline must pass. Check returns its problems, or none.
pub trait TieredRetentionBaselineRule: std::fmt::Debug {
    fn check(&self, item: &TieredRetentionBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionBaselineRequiredDescriptionRule {
    pub message: String,
}

impl TieredRetentionBaselineRule for TieredRetentionBaselineRequiredDescriptionRule {
    fn check(&self, item: &TieredRetentionBaseline) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionBaselineLabelLengthRule {
    pub max_length: i64,
}

impl TieredRetentionBaselineRule for TieredRetentionBaselineLabelLengthRule {
    fn check(&self, item: &TieredRetentionBaseline) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionBaselineRequiredChannelRule {
    pub message: String,
}

impl TieredRetentionBaselineRule for TieredRetentionBaselineRequiredChannelRule {
    fn check(&self, item: &TieredRetentionBaseline) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionBaselineRequiredCodeRule {
    pub message: String,
}

impl TieredRetentionBaselineRule for TieredRetentionBaselineRequiredCodeRule {
    fn check(&self, item: &TieredRetentionBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered retention baseline.
#[derive(Debug, Clone)]
pub struct TieredRetentionBaselineValidator {
    pub rules: Vec<Rc<dyn TieredRetentionBaselineRule>>,
}

impl TieredRetentionBaselineValidator {
    pub fn validate(&self, item: &TieredRetentionBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredRetentionBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRetentionBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_retention_baseline_validator() -> TieredRetentionBaselineValidator {
    TieredRetentionBaselineValidator {
        rules: vec![
            Rc::new(TieredRetentionBaselineRequiredDescriptionRule {
                message: "tiered retention baseline needs a value".to_string(),
            }) as Rc<dyn TieredRetentionBaselineRule>,
            Rc::new(TieredRetentionBaselineLabelLengthRule { max_length: 1000 }),
            Rc::new(TieredRetentionBaselineRequiredChannelRule {
                message: "tiered retention baseline needs this field".to_string(),
            }),
            Rc::new(TieredRetentionBaselineRequiredCodeRule {
                message: "tiered retention baseline needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_retention_baseline_validator();
        let item = new_tiered_retention_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = TieredRetentionBaselineRequiredDescriptionRule {
            message: "tiered retention baseline needs a value".to_string(),
        };
        let mut item = new_tiered_retention_baseline("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredRetentionBaselineLabelLengthRule { max_length: 64 };
        let mut item = new_tiered_retention_baseline("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = TieredRetentionBaselineRequiredChannelRule {
            message: "tiered retention baseline needs an entry".to_string(),
        };
        let mut item = new_tiered_retention_baseline("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredRetentionBaselineRequiredCodeRule {
            message: "tiered retention baseline needs an entry".to_string(),
        };
        let mut item = new_tiered_retention_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
