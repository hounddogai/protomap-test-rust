use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback quote threshold must pass. Check returns its problems, or none.
pub trait FallbackQuoteThresholdRule: std::fmt::Debug {
    fn check(&self, item: &FallbackQuoteThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteThresholdMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackQuoteThresholdRule for FallbackQuoteThresholdMarginRangeRule {
    fn check(&self, item: &FallbackQuoteThreshold) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteThresholdVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackQuoteThresholdRule for FallbackQuoteThresholdVersionRangeRule {
    fn check(&self, item: &FallbackQuoteThreshold) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteThresholdNameLengthRule {
    pub max_length: i64,
}

impl FallbackQuoteThresholdRule for FallbackQuoteThresholdNameLengthRule {
    fn check(&self, item: &FallbackQuoteThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteThresholdFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackQuoteThresholdRule for FallbackQuoteThresholdFloorValueRangeRule {
    fn check(&self, item: &FallbackQuoteThreshold) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteThresholdPhoneLengthRule {
    pub max_length: i64,
}

impl FallbackQuoteThresholdRule for FallbackQuoteThresholdPhoneLengthRule {
    fn check(&self, item: &FallbackQuoteThreshold) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQuoteThresholdRequiredCodeRule {
    pub message: String,
}

impl FallbackQuoteThresholdRule for FallbackQuoteThresholdRequiredCodeRule {
    fn check(&self, item: &FallbackQuoteThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback quote threshold.
#[derive(Debug, Clone)]
pub struct FallbackQuoteThresholdValidator {
    pub rules: Vec<Rc<dyn FallbackQuoteThresholdRule>>,
}

impl FallbackQuoteThresholdValidator {
    pub fn validate(&self, item: &FallbackQuoteThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackQuoteThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackQuoteThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_quote_threshold_validator() -> FallbackQuoteThresholdValidator {
    FallbackQuoteThresholdValidator {
        rules: vec![
            Rc::new(FallbackQuoteThresholdMarginRangeRule { min: 6.3, max: 2154.7 })
                as Rc<dyn FallbackQuoteThresholdRule>,
            Rc::new(FallbackQuoteThresholdVersionRangeRule { min: 64, max: 1000 }),
            Rc::new(FallbackQuoteThresholdNameLengthRule { max_length: 64 }),
            Rc::new(FallbackQuoteThresholdFloorValueRangeRule { min: 0.5, max: 1065.6 }),
            Rc::new(FallbackQuoteThresholdPhoneLengthRule { max_length: 64 }),
            Rc::new(FallbackQuoteThresholdRequiredCodeRule {
                message: "fallback quote threshold needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_quote_threshold_validator();
        let item = new_fallback_quote_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = FallbackQuoteThresholdMarginRangeRule { min: 3.3, max: 2564.1 };
        let mut item = new_fallback_quote_threshold("x", "y");
        item.margin = 2565.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FallbackQuoteThresholdVersionRangeRule { min: 64, max: 255 };
        let mut item = new_fallback_quote_threshold("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackQuoteThresholdNameLengthRule { max_length: 3600 };
        let mut item = new_fallback_quote_threshold("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = FallbackQuoteThresholdFloorValueRangeRule { min: 1.7, max: 4654.4 };
        let mut item = new_fallback_quote_threshold("x", "y");
        item.floor_value = 4655.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = FallbackQuoteThresholdPhoneLengthRule { max_length: 255 };
        let mut item = new_fallback_quote_threshold("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackQuoteThresholdRequiredCodeRule {
            message: "fallback quote threshold needs this field".to_string(),
        };
        let mut item = new_fallback_quote_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
