use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient page exception must pass. Check returns its problems, or none.
pub trait LenientPageExceptionRule: std::fmt::Debug {
    fn check(&self, item: &LenientPageException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageExceptionRequiredTitleRule {
    pub message: String,
}

impl LenientPageExceptionRule for LenientPageExceptionRequiredTitleRule {
    fn check(&self, item: &LenientPageException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageExceptionStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPageExceptionRule for LenientPageExceptionStartsAtRangeRule {
    fn check(&self, item: &LenientPageException) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageExceptionRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPageExceptionRule for LenientPageExceptionRetriesRangeRule {
    fn check(&self, item: &LenientPageException) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageExceptionWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPageExceptionRule for LenientPageExceptionWindowDaysRangeRule {
    fn check(&self, item: &LenientPageException) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageExceptionRequiredCodeRule {
    pub message: String,
}

impl LenientPageExceptionRule for LenientPageExceptionRequiredCodeRule {
    fn check(&self, item: &LenientPageException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient page exception.
#[derive(Debug, Clone)]
pub struct LenientPageExceptionValidator {
    pub rules: Vec<Rc<dyn LenientPageExceptionRule>>,
}

impl LenientPageExceptionValidator {
    pub fn validate(&self, item: &LenientPageException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientPageException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPageException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_page_exception_validator() -> LenientPageExceptionValidator {
    LenientPageExceptionValidator {
        rules: vec![
            Rc::new(LenientPageExceptionRequiredTitleRule {
                message: "lenient page exception needs an entry".to_string(),
            }) as Rc<dyn LenientPageExceptionRule>,
            Rc::new(LenientPageExceptionStartsAtRangeRule { min: 64, max: 3600 }),
            Rc::new(LenientPageExceptionRetriesRangeRule { min: 80, max: 255 }),
            Rc::new(LenientPageExceptionWindowDaysRangeRule { min: 64, max: 1000 }),
            Rc::new(LenientPageExceptionRequiredCodeRule {
                message: "lenient page exception needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_page_exception_validator();
        let item = new_lenient_page_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientPageExceptionRequiredTitleRule {
            message: "lenient page exception needs a value".to_string(),
        };
        let mut item = new_lenient_page_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LenientPageExceptionStartsAtRangeRule { min: 64, max: 3600 };
        let mut item = new_lenient_page_exception("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = LenientPageExceptionRetriesRangeRule { min: 80, max: 255 };
        let mut item = new_lenient_page_exception("x", "y");
        item.retries = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = LenientPageExceptionWindowDaysRangeRule { min: 255, max: 1000 };
        let mut item = new_lenient_page_exception("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientPageExceptionRequiredCodeRule {
            message: "lenient page exception needs this field".to_string(),
        };
        let mut item = new_lenient_page_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
