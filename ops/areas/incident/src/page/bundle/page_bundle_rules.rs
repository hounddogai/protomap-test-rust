use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page bundle must pass. Check returns its problems, or none.
pub trait PageBundleRule: std::fmt::Debug {
    fn check(&self, item: &PageBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBundleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageBundleRule for PageBundleRateRangeRule {
    fn check(&self, item: &PageBundle) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBundleSummaryLengthRule {
    pub max_length: i64,
}

impl PageBundleRule for PageBundleSummaryLengthRule {
    fn check(&self, item: &PageBundle) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBundleRequiredCodeRule {
    pub message: String,
}

impl PageBundleRule for PageBundleRequiredCodeRule {
    fn check(&self, item: &PageBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBundleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageBundleRule for PageBundleRatioRangeRule {
    fn check(&self, item: &PageBundle) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBundleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PageBundleRule for PageBundleRotationDaysRangeRule {
    fn check(&self, item: &PageBundle) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBundleRequiredNameRule {
    pub message: String,
}

impl PageBundleRule for PageBundleRequiredNameRule {
    fn check(&self, item: &PageBundle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a page bundle.
#[derive(Debug, Clone)]
pub struct PageBundleValidator {
    pub rules: Vec<Rc<dyn PageBundleRule>>,
}

impl PageBundleValidator {
    pub fn validate(&self, item: &PageBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_bundle_validator() -> PageBundleValidator {
    PageBundleValidator {
        rules: vec![
            Rc::new(PageBundleRateRangeRule { min: 9.4, max: 813.0 }) as Rc<dyn PageBundleRule>,
            Rc::new(PageBundleSummaryLengthRule { max_length: 80 }),
            Rc::new(PageBundleRequiredCodeRule {
                message: "page bundle needs a value".to_string(),
            }),
            Rc::new(PageBundleRatioRangeRule { min: 9.6, max: 1544.5 }),
            Rc::new(PageBundleRotationDaysRangeRule { min: 32, max: 32 }),
            Rc::new(PageBundleRequiredNameRule {
                message: "page bundle needs this field".to_string(),
            }),
        ],
    }
}
