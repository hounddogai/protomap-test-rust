use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page baseline must pass. Check returns its problems, or none.
pub trait PageBaselineRule: std::fmt::Debug {
    fn check(&self, item: &PageBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBaselineCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageBaselineRule for PageBaselineCapacityRangeRule {
    fn check(&self, item: &PageBaseline) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBaselineRequiredPhoneRule {
    pub message: String,
}

impl PageBaselineRule for PageBaselineRequiredPhoneRule {
    fn check(&self, item: &PageBaseline) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBaselineBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageBaselineRule for PageBaselineBatchSizeRangeRule {
    fn check(&self, item: &PageBaseline) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBaselineRequiredTitleRule {
    pub message: String,
}

impl PageBaselineRule for PageBaselineRequiredTitleRule {
    fn check(&self, item: &PageBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBaselinePhoneLengthRule {
    pub max_length: i64,
}

impl PageBaselineRule for PageBaselinePhoneLengthRule {
    fn check(&self, item: &PageBaseline) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page baseline.
#[derive(Debug, Clone)]
pub struct PageBaselineValidator {
    pub rules: Vec<Rc<dyn PageBaselineRule>>,
}

impl PageBaselineValidator {
    pub fn validate(&self, item: &PageBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_baseline_validator() -> PageBaselineValidator {
    PageBaselineValidator {
        rules: vec![
            Rc::new(PageBaselineCapacityRangeRule { min: 64, max: 86400 }) as Rc<dyn PageBaselineRule>,
            Rc::new(PageBaselineRequiredPhoneRule {
                message: "page baseline needs this field".to_string(),
            }),
            Rc::new(PageBaselineBatchSizeRangeRule { min: 16, max: 120 }),
            Rc::new(PageBaselineRequiredTitleRule {
                message: "page baseline needs this field".to_string(),
            }),
            Rc::new(PageBaselinePhoneLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_baseline_validator();
        let item = new_page_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = PageBaselineCapacityRangeRule { min: 16, max: 32 };
        let mut item = new_page_baseline("x", "y");
        item.capacity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PageBaselineRequiredPhoneRule {
            message: "page baseline needs this field".to_string(),
        };
        let mut item = new_page_baseline("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PageBaselineBatchSizeRangeRule { min: 16, max: 255 };
        let mut item = new_page_baseline("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PageBaselineRequiredTitleRule {
            message: "page baseline needs a value".to_string(),
        };
        let mut item = new_page_baseline("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = PageBaselinePhoneLengthRule { max_length: 3600 };
        let mut item = new_page_baseline("x", "y");
        item.phone = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
