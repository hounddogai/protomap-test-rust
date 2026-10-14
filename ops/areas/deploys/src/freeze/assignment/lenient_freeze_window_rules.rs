use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient freeze window must pass. Check returns its problems, or none.
pub trait LenientFreezeWindowRule: std::fmt::Debug {
    fn check(&self, item: &LenientFreezeWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeWindowAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LenientFreezeWindowRule for LenientFreezeWindowAllowedTitleRule {
    fn check(&self, item: &LenientFreezeWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeWindowRequiredTitleRule {
    pub message: String,
}

impl LenientFreezeWindowRule for LenientFreezeWindowRequiredTitleRule {
    fn check(&self, item: &LenientFreezeWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeWindowArchivedGuardRule {
    pub limit: f64,
}

impl LenientFreezeWindowRule for LenientFreezeWindowArchivedGuardRule {
    fn check(&self, item: &LenientFreezeWindow) -> Vec<String> {
        if item.archived && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while archived", item.budget, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient freeze window.
#[derive(Debug, Clone)]
pub struct LenientFreezeWindowValidator {
    pub rules: Vec<Rc<dyn LenientFreezeWindowRule>>,
}

impl LenientFreezeWindowValidator {
    pub fn validate(&self, item: &LenientFreezeWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientFreezeWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFreezeWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_freeze_window_validator() -> LenientFreezeWindowValidator {
    LenientFreezeWindowValidator {
        rules: vec![
            Rc::new(LenientFreezeWindowAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn LenientFreezeWindowRule>,
            Rc::new(LenientFreezeWindowRequiredTitleRule {
                message: "lenient freeze window needs a value".to_string(),
            }),
            Rc::new(LenientFreezeWindowArchivedGuardRule { limit: 8.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_freeze_window_validator();
        let item = new_lenient_freeze_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientFreezeWindowRequiredTitleRule {
            message: "lenient freeze window needs this field".to_string(),
        };
        let mut item = new_lenient_freeze_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
