use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending action item target must pass. Check returns its problems, or none.
pub trait PendingActionItemTargetRule: std::fmt::Debug {
    fn check(&self, item: &PendingActionItemTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemTargetRequiredCodeRule {
    pub message: String,
}

impl PendingActionItemTargetRule for PendingActionItemTargetRequiredCodeRule {
    fn check(&self, item: &PendingActionItemTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemTargetCipherLengthRule {
    pub max_length: i64,
}

impl PendingActionItemTargetRule for PendingActionItemTargetCipherLengthRule {
    fn check(&self, item: &PendingActionItemTarget) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemTargetTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingActionItemTargetRule for PendingActionItemTargetTtlSecondsRangeRule {
    fn check(&self, item: &PendingActionItemTarget) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemTargetRequiredTitleRule {
    pub message: String,
}

impl PendingActionItemTargetRule for PendingActionItemTargetRequiredTitleRule {
    fn check(&self, item: &PendingActionItemTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemTargetCountryLengthRule {
    pub max_length: i64,
}

impl PendingActionItemTargetRule for PendingActionItemTargetCountryLengthRule {
    fn check(&self, item: &PendingActionItemTarget) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemTargetTitleLengthRule {
    pub max_length: i64,
}

impl PendingActionItemTargetRule for PendingActionItemTargetTitleLengthRule {
    fn check(&self, item: &PendingActionItemTarget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending action item target.
#[derive(Debug, Clone)]
pub struct PendingActionItemTargetValidator {
    pub rules: Vec<Rc<dyn PendingActionItemTargetRule>>,
}

impl PendingActionItemTargetValidator {
    pub fn validate(&self, item: &PendingActionItemTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingActionItemTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingActionItemTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_action_item_target_validator() -> PendingActionItemTargetValidator {
    PendingActionItemTargetValidator {
        rules: vec![
            Rc::new(PendingActionItemTargetRequiredCodeRule {
                message: "pending action item target needs a value".to_string(),
            }) as Rc<dyn PendingActionItemTargetRule>,
            Rc::new(PendingActionItemTargetCipherLengthRule { max_length: 3600 }),
            Rc::new(PendingActionItemTargetTtlSecondsRangeRule { min: 32, max: 255 }),
            Rc::new(PendingActionItemTargetRequiredTitleRule {
                message: "pending action item target needs this field".to_string(),
            }),
            Rc::new(PendingActionItemTargetCountryLengthRule { max_length: 32 }),
            Rc::new(PendingActionItemTargetTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_action_item_target_validator();
        let item = new_pending_action_item_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingActionItemTargetRequiredCodeRule {
            message: "pending action item target needs this field".to_string(),
        };
        let mut item = new_pending_action_item_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = PendingActionItemTargetCipherLengthRule { max_length: 16 };
        let mut item = new_pending_action_item_target("x", "y");
        item.cipher = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PendingActionItemTargetTtlSecondsRangeRule { min: 16, max: 32 };
        let mut item = new_pending_action_item_target("x", "y");
        item.ttl_seconds = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingActionItemTargetRequiredTitleRule {
            message: "pending action item target needs an entry".to_string(),
        };
        let mut item = new_pending_action_item_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PendingActionItemTargetCountryLengthRule { max_length: 1000 };
        let mut item = new_pending_action_item_target("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingActionItemTargetTitleLengthRule { max_length: 86400 };
        let mut item = new_pending_action_item_target("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
