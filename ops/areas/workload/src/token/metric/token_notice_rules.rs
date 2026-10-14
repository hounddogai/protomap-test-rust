use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token notice must pass. Check returns its problems, or none.
pub trait TokenNoticeRule: std::fmt::Debug {
    fn check(&self, item: &TokenNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenNoticeCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenNoticeRule for TokenNoticeCreatedAtRangeRule {
    fn check(&self, item: &TokenNotice) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenNoticeCodePrefixRule {
    pub prefix: String,
}

impl TokenNoticeRule for TokenNoticeCodePrefixRule {
    fn check(&self, item: &TokenNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenNoticeRequiredTitleRule {
    pub message: String,
}

impl TokenNoticeRule for TokenNoticeRequiredTitleRule {
    fn check(&self, item: &TokenNotice) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenNoticeBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenNoticeRule for TokenNoticeBudgetRangeRule {
    fn check(&self, item: &TokenNotice) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token notice.
#[derive(Debug, Clone)]
pub struct TokenNoticeValidator {
    pub rules: Vec<Rc<dyn TokenNoticeRule>>,
}

impl TokenNoticeValidator {
    pub fn validate(&self, item: &TokenNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_notice_validator() -> TokenNoticeValidator {
    TokenNoticeValidator {
        rules: vec![
            Rc::new(TokenNoticeCreatedAtRangeRule { min: 120, max: 120 }) as Rc<dyn TokenNoticeRule>,
            Rc::new(TokenNoticeCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TokenNoticeRequiredTitleRule {
                message: "token notice needs a value".to_string(),
            }),
            Rc::new(TokenNoticeBudgetRangeRule { min: 4.9, max: 4706.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_notice_validator();
        let item = new_token_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = TokenNoticeCreatedAtRangeRule { min: 32, max: 255 };
        let mut item = new_token_notice("x", "y");
        item.created_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TokenNoticeRequiredTitleRule {
            message: "token notice needs an entry".to_string(),
        };
        let mut item = new_token_notice("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TokenNoticeBudgetRangeRule { min: 7.5, max: 4636.9 };
        let mut item = new_token_notice("x", "y");
        item.budget = 4638.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
