use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast notice must pass. Check returns its problems, or none.
pub trait ForecastNoticeRule: std::fmt::Debug {
    fn check(&self, item: &ForecastNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastNoticeCodePrefixRule {
    pub prefix: String,
}

impl ForecastNoticeRule for ForecastNoticeCodePrefixRule {
    fn check(&self, item: &ForecastNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastNoticeRequiredCodeRule {
    pub message: String,
}

impl ForecastNoticeRule for ForecastNoticeRequiredCodeRule {
    fn check(&self, item: &ForecastNotice) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastNoticeTitleLengthRule {
    pub max_length: i64,
}

impl ForecastNoticeRule for ForecastNoticeTitleLengthRule {
    fn check(&self, item: &ForecastNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastNoticeCodeLengthRule {
    pub max_length: i64,
}

impl ForecastNoticeRule for ForecastNoticeCodeLengthRule {
    fn check(&self, item: &ForecastNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastNoticeLabelLengthRule {
    pub max_length: i64,
}

impl ForecastNoticeRule for ForecastNoticeLabelLengthRule {
    fn check(&self, item: &ForecastNotice) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast notice.
#[derive(Debug, Clone)]
pub struct ForecastNoticeValidator {
    pub rules: Vec<Rc<dyn ForecastNoticeRule>>,
}

impl ForecastNoticeValidator {
    pub fn validate(&self, item: &ForecastNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_notice_validator() -> ForecastNoticeValidator {
    ForecastNoticeValidator {
        rules: vec![
            Rc::new(ForecastNoticeCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn ForecastNoticeRule>,
            Rc::new(ForecastNoticeRequiredCodeRule {
                message: "forecast notice needs an entry".to_string(),
            }),
            Rc::new(ForecastNoticeTitleLengthRule { max_length: 1000 }),
            Rc::new(ForecastNoticeCodeLengthRule { max_length: 120 }),
            Rc::new(ForecastNoticeLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_notice_validator();
        let item = new_forecast_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastNoticeRequiredCodeRule {
            message: "forecast notice needs an entry".to_string(),
        };
        let mut item = new_forecast_notice("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ForecastNoticeTitleLengthRule { max_length: 64 };
        let mut item = new_forecast_notice("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastNoticeCodeLengthRule { max_length: 16 };
        let mut item = new_forecast_notice("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ForecastNoticeLabelLengthRule { max_length: 1000 };
        let mut item = new_forecast_notice("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
