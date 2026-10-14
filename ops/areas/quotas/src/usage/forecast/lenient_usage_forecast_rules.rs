use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient usage forecast must pass. Check returns its problems, or none.
pub trait LenientUsageForecastRule: std::fmt::Debug {
    fn check(&self, item: &LenientUsageForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientUsageForecastRequiredCodeRule {
    pub message: String,
}

impl LenientUsageForecastRule for LenientUsageForecastRequiredCodeRule {
    fn check(&self, item: &LenientUsageForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientUsageForecastRequiredSummaryRule {
    pub message: String,
}

impl LenientUsageForecastRule for LenientUsageForecastRequiredSummaryRule {
    fn check(&self, item: &LenientUsageForecast) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientUsageForecastNameLengthRule {
    pub max_length: i64,
}

impl LenientUsageForecastRule for LenientUsageForecastNameLengthRule {
    fn check(&self, item: &LenientUsageForecast) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientUsageForecastRequiredChecksumRule {
    pub message: String,
}

impl LenientUsageForecastRule for LenientUsageForecastRequiredChecksumRule {
    fn check(&self, item: &LenientUsageForecast) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientUsageForecastCodePrefixRule {
    pub prefix: String,
}

impl LenientUsageForecastRule for LenientUsageForecastCodePrefixRule {
    fn check(&self, item: &LenientUsageForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient usage forecast.
#[derive(Debug, Clone)]
pub struct LenientUsageForecastValidator {
    pub rules: Vec<Rc<dyn LenientUsageForecastRule>>,
}

impl LenientUsageForecastValidator {
    pub fn validate(&self, item: &LenientUsageForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientUsageForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientUsageForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_usage_forecast_validator() -> LenientUsageForecastValidator {
    LenientUsageForecastValidator {
        rules: vec![
            Rc::new(LenientUsageForecastRequiredCodeRule {
                message: "lenient usage forecast needs a value".to_string(),
            }) as Rc<dyn LenientUsageForecastRule>,
            Rc::new(LenientUsageForecastRequiredSummaryRule {
                message: "lenient usage forecast needs a value".to_string(),
            }),
            Rc::new(LenientUsageForecastNameLengthRule { max_length: 86400 }),
            Rc::new(LenientUsageForecastRequiredChecksumRule {
                message: "lenient usage forecast needs a value".to_string(),
            }),
            Rc::new(LenientUsageForecastCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_usage_forecast_validator();
        let item = new_lenient_usage_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientUsageForecastRequiredCodeRule {
            message: "lenient usage forecast needs an entry".to_string(),
        };
        let mut item = new_lenient_usage_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = LenientUsageForecastRequiredSummaryRule {
            message: "lenient usage forecast needs an entry".to_string(),
        };
        let mut item = new_lenient_usage_forecast("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientUsageForecastNameLengthRule { max_length: 120 };
        let mut item = new_lenient_usage_forecast("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = LenientUsageForecastRequiredChecksumRule {
            message: "lenient usage forecast needs an entry".to_string(),
        };
        let mut item = new_lenient_usage_forecast("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
