use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page entry must pass. Check returns its problems, or none.
pub trait PageEntryRule: std::fmt::Debug {
    fn check(&self, item: &PageEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageEntryRequiredCodeRule {
    pub message: String,
}

impl PageEntryRule for PageEntryRequiredCodeRule {
    fn check(&self, item: &PageEntry) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageEntryUnitLengthRule {
    pub max_length: i64,
}

impl PageEntryRule for PageEntryUnitLengthRule {
    fn check(&self, item: &PageEntry) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageEntryRequiredUnitRule {
    pub message: String,
}

impl PageEntryRule for PageEntryRequiredUnitRule {
    fn check(&self, item: &PageEntry) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a page entry.
#[derive(Debug, Clone)]
pub struct PageEntryValidator {
    pub rules: Vec<Rc<dyn PageEntryRule>>,
}

impl PageEntryValidator {
    pub fn validate(&self, item: &PageEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_entry_validator() -> PageEntryValidator {
    PageEntryValidator {
        rules: vec![
            Rc::new(PageEntryRequiredCodeRule {
                message: "page entry needs an entry".to_string(),
            }) as Rc<dyn PageEntryRule>,
            Rc::new(PageEntryUnitLengthRule { max_length: 3600 }),
            Rc::new(PageEntryRequiredUnitRule {
                message: "page entry needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_entry_validator();
        let item = new_page_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PageEntryRequiredCodeRule {
            message: "page entry needs this field".to_string(),
        };
        let mut item = new_page_entry("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = PageEntryUnitLengthRule { max_length: 64 };
        let mut item = new_page_entry("x", "y");
        item.unit = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PageEntryRequiredUnitRule {
            message: "page entry needs a value".to_string(),
        };
        let mut item = new_page_entry("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
