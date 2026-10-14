use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted data export notice must pass. Check returns its problems, or none.
pub trait WeightedDataExportNoticeRule: std::fmt::Debug {
    fn check(&self, item: &WeightedDataExportNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportNoticeAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl WeightedDataExportNoticeRule for WeightedDataExportNoticeAllowedCountryRule {
    fn check(&self, item: &WeightedDataExportNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is inconsistent", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportNoticeRequiredPhoneRule {
    pub message: String,
}

impl WeightedDataExportNoticeRule for WeightedDataExportNoticeRequiredPhoneRule {
    fn check(&self, item: &WeightedDataExportNotice) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportNoticeCodeLengthRule {
    pub max_length: i64,
}

impl WeightedDataExportNoticeRule for WeightedDataExportNoticeCodeLengthRule {
    fn check(&self, item: &WeightedDataExportNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportNoticeDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedDataExportNoticeRule for WeightedDataExportNoticeDiscountRangeRule {
    fn check(&self, item: &WeightedDataExportNotice) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportNoticeRequiredCountryRule {
    pub message: String,
}

impl WeightedDataExportNoticeRule for WeightedDataExportNoticeRequiredCountryRule {
    fn check(&self, item: &WeightedDataExportNotice) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedDataExportNoticeRequiredNameRule {
    pub message: String,
}

impl WeightedDataExportNoticeRule for WeightedDataExportNoticeRequiredNameRule {
    fn check(&self, item: &WeightedDataExportNotice) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted data export notice.
#[derive(Debug, Clone)]
pub struct WeightedDataExportNoticeValidator {
    pub rules: Vec<Rc<dyn WeightedDataExportNoticeRule>>,
}

impl WeightedDataExportNoticeValidator {
    pub fn validate(&self, item: &WeightedDataExportNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedDataExportNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedDataExportNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_data_export_notice_validator() -> WeightedDataExportNoticeValidator {
    WeightedDataExportNoticeValidator {
        rules: vec![
            Rc::new(WeightedDataExportNoticeAllowedCountryRule {
                allowed: vec!["adaptive".to_string()],
            }) as Rc<dyn WeightedDataExportNoticeRule>,
            Rc::new(WeightedDataExportNoticeRequiredPhoneRule {
                message: "weighted data export notice needs this field".to_string(),
            }),
            Rc::new(WeightedDataExportNoticeCodeLengthRule { max_length: 1000 }),
            Rc::new(WeightedDataExportNoticeDiscountRangeRule { min: 7.3, max: 4356.1 }),
            Rc::new(WeightedDataExportNoticeRequiredCountryRule {
                message: "weighted data export notice needs a value".to_string(),
            }),
            Rc::new(WeightedDataExportNoticeRequiredNameRule {
                message: "weighted data export notice needs a value".to_string(),
            }),
        ],
    }
}
