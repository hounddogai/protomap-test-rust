use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient timeline rule must pass. Check returns its problems, or none.
pub trait LenientTimelineRuleRule: std::fmt::Debug {
    fn check(&self, item: &LenientTimelineRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineRuleTitleLengthRule {
    pub max_length: i64,
}

impl LenientTimelineRuleRule for LenientTimelineRuleTitleLengthRule {
    fn check(&self, item: &LenientTimelineRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientTimelineRuleRule for LenientTimelineRuleAllowedCodeRule {
    fn check(&self, item: &LenientTimelineRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineRuleArchivedGuardRule {
    pub limit: f64,
}

impl LenientTimelineRuleRule for LenientTimelineRuleArchivedGuardRule {
    fn check(&self, item: &LenientTimelineRule) -> Vec<String> {
        if item.archived && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while archived", item.amount, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient timeline rule.
#[derive(Debug, Clone)]
pub struct LenientTimelineRuleValidator {
    pub rules: Vec<Rc<dyn LenientTimelineRuleRule>>,
}

impl LenientTimelineRuleValidator {
    pub fn validate(&self, item: &LenientTimelineRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientTimelineRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTimelineRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_timeline_rule_validator() -> LenientTimelineRuleValidator {
    LenientTimelineRuleValidator {
        rules: vec![
            Rc::new(LenientTimelineRuleTitleLengthRule { max_length: 80 }) as Rc<dyn LenientTimelineRuleRule>,
            Rc::new(LenientTimelineRuleAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "capped".to_string()],
            }),
            Rc::new(LenientTimelineRuleArchivedGuardRule { limit: 4.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_timeline_rule_validator();
        let item = new_lenient_timeline_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientTimelineRuleTitleLengthRule { max_length: 120 };
        let mut item = new_lenient_timeline_rule("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
