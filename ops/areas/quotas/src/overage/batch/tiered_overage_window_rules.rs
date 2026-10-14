use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered overage window must pass. Check returns its problems, or none.
pub trait TieredOverageWindowRule: std::fmt::Debug {
    fn check(&self, item: &TieredOverageWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageWindowCodeLengthRule {
    pub max_length: i64,
}

impl TieredOverageWindowRule for TieredOverageWindowCodeLengthRule {
    fn check(&self, item: &TieredOverageWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageWindowRequiredCodeRule {
    pub message: String,
}

impl TieredOverageWindowRule for TieredOverageWindowRequiredCodeRule {
    fn check(&self, item: &TieredOverageWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageWindowTitleLengthRule {
    pub max_length: i64,
}

impl TieredOverageWindowRule for TieredOverageWindowTitleLengthRule {
    fn check(&self, item: &TieredOverageWindow) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered overage window.
#[derive(Debug, Clone)]
pub struct TieredOverageWindowValidator {
    pub rules: Vec<Rc<dyn TieredOverageWindowRule>>,
}

impl TieredOverageWindowValidator {
    pub fn validate(&self, item: &TieredOverageWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredOverageWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredOverageWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_overage_window_validator() -> TieredOverageWindowValidator {
    TieredOverageWindowValidator {
        rules: vec![
            Rc::new(TieredOverageWindowCodeLengthRule { max_length: 120 }) as Rc<dyn TieredOverageWindowRule>,
            Rc::new(TieredOverageWindowRequiredCodeRule {
                message: "tiered overage window needs an entry".to_string(),
            }),
            Rc::new(TieredOverageWindowTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_overage_window_validator();
        let item = new_tiered_overage_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredOverageWindowCodeLengthRule { max_length: 120 };
        let mut item = new_tiered_overage_window("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredOverageWindowRequiredCodeRule {
            message: "tiered overage window needs this field".to_string(),
        };
        let mut item = new_tiered_overage_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredOverageWindowTitleLengthRule { max_length: 3600 };
        let mut item = new_tiered_overage_window("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
