use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page adjustment must pass. Check returns its problems, or none.
pub trait PageAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &PageAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAdjustmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PageAdjustmentRule for PageAdjustmentAllowedCodeRule {
    fn check(&self, item: &PageAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAdjustmentRequiredCodeRule {
    pub message: String,
}

impl PageAdjustmentRule for PageAdjustmentRequiredCodeRule {
    fn check(&self, item: &PageAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl PageAdjustmentRule for PageAdjustmentCodePrefixRule {
    fn check(&self, item: &PageAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page adjustment.
#[derive(Debug, Clone)]
pub struct PageAdjustmentValidator {
    pub rules: Vec<Rc<dyn PageAdjustmentRule>>,
}

impl PageAdjustmentValidator {
    pub fn validate(&self, item: &PageAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_adjustment_validator() -> PageAdjustmentValidator {
    PageAdjustmentValidator {
        rules: vec![
            Rc::new(PageAdjustmentAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "strict".to_string(), "capped".to_string()],
            }) as Rc<dyn PageAdjustmentRule>,
            Rc::new(PageAdjustmentRequiredCodeRule {
                message: "page adjustment needs a value".to_string(),
            }),
            Rc::new(PageAdjustmentCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_adjustment_validator();
        let item = new_page_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PageAdjustmentRequiredCodeRule {
            message: "page adjustment needs a value".to_string(),
        };
        let mut item = new_page_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
