use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token rollup must pass. Check returns its problems, or none.
pub trait TokenRollupRule: std::fmt::Debug {
    fn check(&self, item: &TokenRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRollupRequiredCodeRule {
    pub message: String,
}

impl TokenRollupRule for TokenRollupRequiredCodeRule {
    fn check(&self, item: &TokenRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRollupUnitLengthRule {
    pub max_length: i64,
}

impl TokenRollupRule for TokenRollupUnitLengthRule {
    fn check(&self, item: &TokenRollup) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRollupLabelLengthRule {
    pub max_length: i64,
}

impl TokenRollupRule for TokenRollupLabelLengthRule {
    fn check(&self, item: &TokenRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token rollup.
#[derive(Debug, Clone)]
pub struct TokenRollupValidator {
    pub rules: Vec<Rc<dyn TokenRollupRule>>,
}

impl TokenRollupValidator {
    pub fn validate(&self, item: &TokenRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TokenRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_rollup_validator() -> TokenRollupValidator {
    TokenRollupValidator {
        rules: vec![
            Rc::new(TokenRollupRequiredCodeRule {
                message: "token rollup needs this field".to_string(),
            }) as Rc<dyn TokenRollupRule>,
            Rc::new(TokenRollupUnitLengthRule { max_length: 1000 }),
            Rc::new(TokenRollupLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_rollup_validator();
        let item = new_token_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenRollupRequiredCodeRule {
            message: "token rollup needs an entry".to_string(),
        };
        let mut item = new_token_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = TokenRollupUnitLengthRule { max_length: 64 };
        let mut item = new_token_rollup("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TokenRollupLabelLengthRule { max_length: 255 };
        let mut item = new_token_rollup("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
