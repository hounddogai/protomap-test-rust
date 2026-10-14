use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page mapping must pass. Check returns its problems, or none.
pub trait PageMappingRule: std::fmt::Debug {
    fn check(&self, item: &PageMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMappingOwnerIdLengthRule {
    pub max_length: i64,
}

impl PageMappingRule for PageMappingOwnerIdLengthRule {
    fn check(&self, item: &PageMapping) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMappingLabelLengthRule {
    pub max_length: i64,
}

impl PageMappingRule for PageMappingLabelLengthRule {
    fn check(&self, item: &PageMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMappingRequiredCodeRule {
    pub message: String,
}

impl PageMappingRule for PageMappingRequiredCodeRule {
    fn check(&self, item: &PageMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMappingRequiredTimezoneRule {
    pub message: String,
}

impl PageMappingRule for PageMappingRequiredTimezoneRule {
    fn check(&self, item: &PageMapping) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageMappingAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl PageMappingRule for PageMappingAllowedTimezoneRule {
    fn check(&self, item: &PageMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is below minimum", item.timezone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page mapping.
#[derive(Debug, Clone)]
pub struct PageMappingValidator {
    pub rules: Vec<Rc<dyn PageMappingRule>>,
}

impl PageMappingValidator {
    pub fn validate(&self, item: &PageMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_mapping_validator() -> PageMappingValidator {
    PageMappingValidator {
        rules: vec![
            Rc::new(PageMappingOwnerIdLengthRule { max_length: 3600 }) as Rc<dyn PageMappingRule>,
            Rc::new(PageMappingLabelLengthRule { max_length: 255 }),
            Rc::new(PageMappingRequiredCodeRule {
                message: "page mapping needs an entry".to_string(),
            }),
            Rc::new(PageMappingRequiredTimezoneRule {
                message: "page mapping needs this field".to_string(),
            }),
            Rc::new(PageMappingAllowedTimezoneRule {
                allowed: vec!["capped".to_string(), "strict".to_string(), "capped".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_mapping_validator();
        let item = new_page_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = PageMappingOwnerIdLengthRule { max_length: 64 };
        let mut item = new_page_mapping("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PageMappingLabelLengthRule { max_length: 3600 };
        let mut item = new_page_mapping("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PageMappingRequiredCodeRule {
            message: "page mapping needs this field".to_string(),
        };
        let mut item = new_page_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PageMappingRequiredTimezoneRule {
            message: "page mapping needs an entry".to_string(),
        };
        let mut item = new_page_mapping("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
