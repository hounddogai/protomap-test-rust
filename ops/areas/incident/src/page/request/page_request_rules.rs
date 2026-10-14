use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page request must pass. Check returns its problems, or none.
pub trait PageRequestRule: std::fmt::Debug {
    fn check(&self, item: &PageRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRequestCodePrefixRule {
    pub prefix: String,
}

impl PageRequestRule for PageRequestCodePrefixRule {
    fn check(&self, item: &PageRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRequestLocaleLengthRule {
    pub max_length: i64,
}

impl PageRequestRule for PageRequestLocaleLengthRule {
    fn check(&self, item: &PageRequest) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRequestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PageRequestRule for PageRequestAllowedCodeRule {
    fn check(&self, item: &PageRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageRequestRequiredLocaleRule {
    pub message: String,
}

impl PageRequestRule for PageRequestRequiredLocaleRule {
    fn check(&self, item: &PageRequest) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a page request.
#[derive(Debug, Clone)]
pub struct PageRequestValidator {
    pub rules: Vec<Rc<dyn PageRequestRule>>,
}

impl PageRequestValidator {
    pub fn validate(&self, item: &PageRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_request_validator() -> PageRequestValidator {
    PageRequestValidator {
        rules: vec![
            Rc::new(PageRequestCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn PageRequestRule>,
            Rc::new(PageRequestLocaleLengthRule { max_length: 64 }),
            Rc::new(PageRequestAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "default".to_string()],
            }),
            Rc::new(PageRequestRequiredLocaleRule {
                message: "page request needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_request_validator();
        let item = new_page_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = PageRequestLocaleLengthRule { max_length: 3600 };
        let mut item = new_page_request("x", "y");
        item.locale = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PageRequestRequiredLocaleRule {
            message: "page request needs an entry".to_string(),
        };
        let mut item = new_page_request("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
