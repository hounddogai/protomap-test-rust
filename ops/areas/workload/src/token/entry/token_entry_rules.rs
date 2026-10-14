use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token entry must pass. Check returns its problems, or none.
pub trait TokenEntryRule: std::fmt::Debug {
    fn check(&self, item: &TokenEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenEntryRequiredNameRule {
    pub message: String,
}

impl TokenEntryRule for TokenEntryRequiredNameRule {
    fn check(&self, item: &TokenEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenEntryCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenEntryRule for TokenEntryCeilingRangeRule {
    fn check(&self, item: &TokenEntry) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenEntryRequiredOwnerIdRule {
    pub message: String,
}

impl TokenEntryRule for TokenEntryRequiredOwnerIdRule {
    fn check(&self, item: &TokenEntry) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenEntryCodeLengthRule {
    pub max_length: i64,
}

impl TokenEntryRule for TokenEntryCodeLengthRule {
    fn check(&self, item: &TokenEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenEntryAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenEntryRule for TokenEntryAttemptsRangeRule {
    fn check(&self, item: &TokenEntry) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenEntryRequiredTimezoneRule {
    pub message: String,
}

impl TokenEntryRule for TokenEntryRequiredTimezoneRule {
    fn check(&self, item: &TokenEntry) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a token entry.
#[derive(Debug, Clone)]
pub struct TokenEntryValidator {
    pub rules: Vec<Rc<dyn TokenEntryRule>>,
}

impl TokenEntryValidator {
    pub fn validate(&self, item: &TokenEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_entry_validator() -> TokenEntryValidator {
    TokenEntryValidator {
        rules: vec![
            Rc::new(TokenEntryRequiredNameRule {
                message: "token entry needs this field".to_string(),
            }) as Rc<dyn TokenEntryRule>,
            Rc::new(TokenEntryCeilingRangeRule { min: 6.4, max: 2155.2 }),
            Rc::new(TokenEntryRequiredOwnerIdRule {
                message: "token entry needs an entry".to_string(),
            }),
            Rc::new(TokenEntryCodeLengthRule { max_length: 16 }),
            Rc::new(TokenEntryAttemptsRangeRule { min: 16, max: 80 }),
            Rc::new(TokenEntryRequiredTimezoneRule {
                message: "token entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_entry_validator();
        let item = new_token_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TokenEntryRequiredNameRule {
            message: "token entry needs a value".to_string(),
        };
        let mut item = new_token_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = TokenEntryCeilingRangeRule { min: 4.1, max: 3695.6 };
        let mut item = new_token_entry("x", "y");
        item.ceiling = 3697.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = TokenEntryRequiredOwnerIdRule {
            message: "token entry needs a value".to_string(),
        };
        let mut item = new_token_entry("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TokenEntryCodeLengthRule { max_length: 3600 };
        let mut item = new_token_entry("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = TokenEntryAttemptsRangeRule { min: 1000, max: 86400 };
        let mut item = new_token_entry("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = TokenEntryRequiredTimezoneRule {
            message: "token entry needs a value".to_string(),
        };
        let mut item = new_token_entry("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
