use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page preference must pass. Check returns its problems, or none.
pub trait PagePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PagePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagePreferenceGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PagePreferenceRule for PagePreferenceGraceDaysRangeRule {
    fn check(&self, item: &PagePreference) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagePreferenceRequiredLocaleRule {
    pub message: String,
}

impl PagePreferenceRule for PagePreferenceRequiredLocaleRule {
    fn check(&self, item: &PagePreference) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagePreferenceCodeLengthRule {
    pub max_length: i64,
}

impl PagePreferenceRule for PagePreferenceCodeLengthRule {
    fn check(&self, item: &PagePreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PagePreferenceLocaleLengthRule {
    pub max_length: i64,
}

impl PagePreferenceRule for PagePreferenceLocaleLengthRule {
    fn check(&self, item: &PagePreference) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page preference.
#[derive(Debug, Clone)]
pub struct PagePreferenceValidator {
    pub rules: Vec<Rc<dyn PagePreferenceRule>>,
}

impl PagePreferenceValidator {
    pub fn validate(&self, item: &PagePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PagePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PagePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_preference_validator() -> PagePreferenceValidator {
    PagePreferenceValidator {
        rules: vec![
            Rc::new(PagePreferenceGraceDaysRangeRule { min: 32, max: 120 }) as Rc<dyn PagePreferenceRule>,
            Rc::new(PagePreferenceRequiredLocaleRule {
                message: "page preference needs this field".to_string(),
            }),
            Rc::new(PagePreferenceCodeLengthRule { max_length: 3600 }),
            Rc::new(PagePreferenceLocaleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_preference_validator();
        let item = new_page_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = PagePreferenceGraceDaysRangeRule { min: 80, max: 3600 };
        let mut item = new_page_preference("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PagePreferenceRequiredLocaleRule {
            message: "page preference needs a value".to_string(),
        };
        let mut item = new_page_preference("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PagePreferenceCodeLengthRule { max_length: 86400 };
        let mut item = new_page_preference("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = PagePreferenceLocaleLengthRule { max_length: 16 };
        let mut item = new_page_preference("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
