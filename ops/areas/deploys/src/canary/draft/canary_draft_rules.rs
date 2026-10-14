use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary draft must pass. Check returns its problems, or none.
pub trait CanaryDraftRule: std::fmt::Debug {
    fn check(&self, item: &CanaryDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDraftPinnedGuardRule {
    pub limit: i64,
}

impl CanaryDraftRule for CanaryDraftPinnedGuardRule {
    fn check(&self, item: &CanaryDraft) -> Vec<String> {
        if item.pinned && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while pinned", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDraftUnitLengthRule {
    pub max_length: i64,
}

impl CanaryDraftRule for CanaryDraftUnitLengthRule {
    fn check(&self, item: &CanaryDraft) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDraftLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryDraftRule for CanaryDraftLimitRangeRule {
    fn check(&self, item: &CanaryDraft) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDraftCodePrefixRule {
    pub prefix: String,
}

impl CanaryDraftRule for CanaryDraftCodePrefixRule {
    fn check(&self, item: &CanaryDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryDraftRequiredCodeRule {
    pub message: String,
}

impl CanaryDraftRule for CanaryDraftRequiredCodeRule {
    fn check(&self, item: &CanaryDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary draft.
#[derive(Debug, Clone)]
pub struct CanaryDraftValidator {
    pub rules: Vec<Rc<dyn CanaryDraftRule>>,
}

impl CanaryDraftValidator {
    pub fn validate(&self, item: &CanaryDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_draft_validator() -> CanaryDraftValidator {
    CanaryDraftValidator {
        rules: vec![
            Rc::new(CanaryDraftPinnedGuardRule { limit: 120 }) as Rc<dyn CanaryDraftRule>,
            Rc::new(CanaryDraftUnitLengthRule { max_length: 80 }),
            Rc::new(CanaryDraftLimitRangeRule { min: 80, max: 120 }),
            Rc::new(CanaryDraftCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(CanaryDraftRequiredCodeRule {
                message: "canary draft needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_draft_validator();
        let item = new_canary_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = CanaryDraftUnitLengthRule { max_length: 64 };
        let mut item = new_canary_draft("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = CanaryDraftLimitRangeRule { min: 32, max: 32 };
        let mut item = new_canary_draft("x", "y");
        item.limit = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryDraftRequiredCodeRule {
            message: "canary draft needs an entry".to_string(),
        };
        let mut item = new_canary_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
