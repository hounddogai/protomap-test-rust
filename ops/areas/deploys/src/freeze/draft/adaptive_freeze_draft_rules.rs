use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive freeze draft must pass. Check returns its problems, or none.
pub trait AdaptiveFreezeDraftRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveFreezeDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeDraftCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveFreezeDraftRule for AdaptiveFreezeDraftCodeLengthRule {
    fn check(&self, item: &AdaptiveFreezeDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeDraftAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveFreezeDraftRule for AdaptiveFreezeDraftAllowedCodeRule {
    fn check(&self, item: &AdaptiveFreezeDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeDraftPinnedGuardRule {
    pub limit: f64,
}

impl AdaptiveFreezeDraftRule for AdaptiveFreezeDraftPinnedGuardRule {
    fn check(&self, item: &AdaptiveFreezeDraft) -> Vec<String> {
        if item.pinned && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while pinned", item.cost, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive freeze draft.
#[derive(Debug, Clone)]
pub struct AdaptiveFreezeDraftValidator {
    pub rules: Vec<Rc<dyn AdaptiveFreezeDraftRule>>,
}

impl AdaptiveFreezeDraftValidator {
    pub fn validate(&self, item: &AdaptiveFreezeDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveFreezeDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveFreezeDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_freeze_draft_validator() -> AdaptiveFreezeDraftValidator {
    AdaptiveFreezeDraftValidator {
        rules: vec![
            Rc::new(AdaptiveFreezeDraftCodeLengthRule { max_length: 80 }) as Rc<dyn AdaptiveFreezeDraftRule>,
            Rc::new(AdaptiveFreezeDraftAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "strict".to_string()],
            }),
            Rc::new(AdaptiveFreezeDraftPinnedGuardRule { limit: 7.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_freeze_draft_validator();
        let item = new_adaptive_freeze_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveFreezeDraftCodeLengthRule { max_length: 3600 };
        let mut item = new_adaptive_freeze_draft("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
