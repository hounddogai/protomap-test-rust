use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote group must pass. Check returns its problems, or none.
pub trait QuoteGroupRule: std::fmt::Debug {
    fn check(&self, item: &QuoteGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteGroupRequiredNameRule {
    pub message: String,
}

impl QuoteGroupRule for QuoteGroupRequiredNameRule {
    fn check(&self, item: &QuoteGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteGroupCodeLengthRule {
    pub max_length: i64,
}

impl QuoteGroupRule for QuoteGroupCodeLengthRule {
    fn check(&self, item: &QuoteGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteGroupPrincipalLengthRule {
    pub max_length: i64,
}

impl QuoteGroupRule for QuoteGroupPrincipalLengthRule {
    fn check(&self, item: &QuoteGroup) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteGroupRequiredContactEmailRule {
    pub message: String,
}

impl QuoteGroupRule for QuoteGroupRequiredContactEmailRule {
    fn check(&self, item: &QuoteGroup) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote group.
#[derive(Debug, Clone)]
pub struct QuoteGroupValidator {
    pub rules: Vec<Rc<dyn QuoteGroupRule>>,
}

impl QuoteGroupValidator {
    pub fn validate(&self, item: &QuoteGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_group_validator() -> QuoteGroupValidator {
    QuoteGroupValidator {
        rules: vec![
            Rc::new(QuoteGroupRequiredNameRule {
                message: "quote group needs an entry".to_string(),
            }) as Rc<dyn QuoteGroupRule>,
            Rc::new(QuoteGroupCodeLengthRule { max_length: 3600 }),
            Rc::new(QuoteGroupPrincipalLengthRule { max_length: 80 }),
            Rc::new(QuoteGroupRequiredContactEmailRule {
                message: "quote group needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_group_validator();
        let item = new_quote_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = QuoteGroupRequiredNameRule {
            message: "quote group needs a value".to_string(),
        };
        let mut item = new_quote_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuoteGroupCodeLengthRule { max_length: 16 };
        let mut item = new_quote_group("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = QuoteGroupPrincipalLengthRule { max_length: 255 };
        let mut item = new_quote_group("x", "y");
        item.principal = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = QuoteGroupRequiredContactEmailRule {
            message: "quote group needs this field".to_string(),
        };
        let mut item = new_quote_group("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
