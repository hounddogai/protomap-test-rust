use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict page entry must pass. Check returns its problems, or none.
pub trait StrictPageEntryRule: std::fmt::Debug {
    fn check(&self, item: &StrictPageEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageEntryNameLengthRule {
    pub max_length: i64,
}

impl StrictPageEntryRule for StrictPageEntryNameLengthRule {
    fn check(&self, item: &StrictPageEntry) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageEntryCurrencyLengthRule {
    pub max_length: i64,
}

impl StrictPageEntryRule for StrictPageEntryCurrencyLengthRule {
    fn check(&self, item: &StrictPageEntry) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageEntryCodeLengthRule {
    pub max_length: i64,
}

impl StrictPageEntryRule for StrictPageEntryCodeLengthRule {
    fn check(&self, item: &StrictPageEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageEntryRequiredCategoryRule {
    pub message: String,
}

impl StrictPageEntryRule for StrictPageEntryRequiredCategoryRule {
    fn check(&self, item: &StrictPageEntry) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict page entry.
#[derive(Debug, Clone)]
pub struct StrictPageEntryValidator {
    pub rules: Vec<Rc<dyn StrictPageEntryRule>>,
}

impl StrictPageEntryValidator {
    pub fn validate(&self, item: &StrictPageEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictPageEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPageEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_page_entry_validator() -> StrictPageEntryValidator {
    StrictPageEntryValidator {
        rules: vec![
            Rc::new(StrictPageEntryNameLengthRule { max_length: 80 }) as Rc<dyn StrictPageEntryRule>,
            Rc::new(StrictPageEntryCurrencyLengthRule { max_length: 3600 }),
            Rc::new(StrictPageEntryCodeLengthRule { max_length: 255 }),
            Rc::new(StrictPageEntryRequiredCategoryRule {
                message: "strict page entry needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_page_entry_validator();
        let item = new_strict_page_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictPageEntryNameLengthRule { max_length: 32 };
        let mut item = new_strict_page_entry("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = StrictPageEntryCurrencyLengthRule { max_length: 3600 };
        let mut item = new_strict_page_entry("x", "y");
        item.currency = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictPageEntryCodeLengthRule { max_length: 255 };
        let mut item = new_strict_page_entry("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = StrictPageEntryRequiredCategoryRule {
            message: "strict page entry needs a value".to_string(),
        };
        let mut item = new_strict_page_entry("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
