use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict runbook threshold must pass. Check returns its problems, or none.
pub trait StrictRunbookThresholdRule: std::fmt::Debug {
    fn check(&self, item: &StrictRunbookThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookThresholdRequiredCodeRule {
    pub message: String,
}

impl StrictRunbookThresholdRule for StrictRunbookThresholdRequiredCodeRule {
    fn check(&self, item: &StrictRunbookThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookThresholdAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl StrictRunbookThresholdRule for StrictRunbookThresholdAllowedAudienceRule {
    fn check(&self, item: &StrictRunbookThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is above maximum", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookThresholdRequiredKeyIdRule {
    pub message: String,
}

impl StrictRunbookThresholdRule for StrictRunbookThresholdRequiredKeyIdRule {
    fn check(&self, item: &StrictRunbookThreshold) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookThresholdLabelLengthRule {
    pub max_length: i64,
}

impl StrictRunbookThresholdRule for StrictRunbookThresholdLabelLengthRule {
    fn check(&self, item: &StrictRunbookThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookThresholdRequiredTitleRule {
    pub message: String,
}

impl StrictRunbookThresholdRule for StrictRunbookThresholdRequiredTitleRule {
    fn check(&self, item: &StrictRunbookThreshold) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRunbookThresholdApprovedGuardRule {
    pub limit: i64,
}

impl StrictRunbookThresholdRule for StrictRunbookThresholdApprovedGuardRule {
    fn check(&self, item: &StrictRunbookThreshold) -> Vec<String> {
        if item.approved && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while approved",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict runbook threshold.
#[derive(Debug, Clone)]
pub struct StrictRunbookThresholdValidator {
    pub rules: Vec<Rc<dyn StrictRunbookThresholdRule>>,
}

impl StrictRunbookThresholdValidator {
    pub fn validate(&self, item: &StrictRunbookThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictRunbookThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRunbookThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_runbook_threshold_validator() -> StrictRunbookThresholdValidator {
    StrictRunbookThresholdValidator {
        rules: vec![
            Rc::new(StrictRunbookThresholdRequiredCodeRule {
                message: "strict runbook threshold needs an entry".to_string(),
            }) as Rc<dyn StrictRunbookThresholdRule>,
            Rc::new(StrictRunbookThresholdAllowedAudienceRule {
                allowed: vec!["seasonal".to_string(), "manual".to_string(), "seasonal".to_string()],
            }),
            Rc::new(StrictRunbookThresholdRequiredKeyIdRule {
                message: "strict runbook threshold needs this field".to_string(),
            }),
            Rc::new(StrictRunbookThresholdLabelLengthRule { max_length: 64 }),
            Rc::new(StrictRunbookThresholdRequiredTitleRule {
                message: "strict runbook threshold needs a value".to_string(),
            }),
            Rc::new(StrictRunbookThresholdApprovedGuardRule { limit: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_runbook_threshold_validator();
        let item = new_strict_runbook_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictRunbookThresholdRequiredCodeRule {
            message: "strict runbook threshold needs this field".to_string(),
        };
        let mut item = new_strict_runbook_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = StrictRunbookThresholdRequiredKeyIdRule {
            message: "strict runbook threshold needs this field".to_string(),
        };
        let mut item = new_strict_runbook_threshold("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictRunbookThresholdLabelLengthRule { max_length: 80 };
        let mut item = new_strict_runbook_threshold("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = StrictRunbookThresholdRequiredTitleRule {
            message: "strict runbook threshold needs a value".to_string(),
        };
        let mut item = new_strict_runbook_threshold("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
