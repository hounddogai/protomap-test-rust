use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page schedule must pass. Check returns its problems, or none.
pub trait PageScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PageSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScheduleRequiredNameRule {
    pub message: String,
}

impl PageScheduleRule for PageScheduleRequiredNameRule {
    fn check(&self, item: &PageSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScheduleRequiredCodeRule {
    pub message: String,
}

impl PageScheduleRule for PageScheduleRequiredCodeRule {
    fn check(&self, item: &PageSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScheduleTimezoneLengthRule {
    pub max_length: i64,
}

impl PageScheduleRule for PageScheduleTimezoneLengthRule {
    fn check(&self, item: &PageSchedule) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScheduleNameLengthRule {
    pub max_length: i64,
}

impl PageScheduleRule for PageScheduleNameLengthRule {
    fn check(&self, item: &PageSchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScheduleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageScheduleRule for PageScheduleScoreRangeRule {
    fn check(&self, item: &PageSchedule) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageScheduleExternalRefLengthRule {
    pub max_length: i64,
}

impl PageScheduleRule for PageScheduleExternalRefLengthRule {
    fn check(&self, item: &PageSchedule) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a page schedule.
#[derive(Debug, Clone)]
pub struct PageScheduleValidator {
    pub rules: Vec<Rc<dyn PageScheduleRule>>,
}

impl PageScheduleValidator {
    pub fn validate(&self, item: &PageSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_schedule_validator() -> PageScheduleValidator {
    PageScheduleValidator {
        rules: vec![
            Rc::new(PageScheduleRequiredNameRule {
                message: "page schedule needs a value".to_string(),
            }) as Rc<dyn PageScheduleRule>,
            Rc::new(PageScheduleRequiredCodeRule {
                message: "page schedule needs a value".to_string(),
            }),
            Rc::new(PageScheduleTimezoneLengthRule { max_length: 120 }),
            Rc::new(PageScheduleNameLengthRule { max_length: 64 }),
            Rc::new(PageScheduleScoreRangeRule { min: 0.3, max: 3805.8 }),
            Rc::new(PageScheduleExternalRefLengthRule { max_length: 3600 }),
        ],
    }
}
