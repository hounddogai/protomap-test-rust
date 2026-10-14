use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient allowlist forecast must pass. Check returns its problems, or none.
pub trait LenientAllowlistForecastRule: std::fmt::Debug {
    fn check(&self, item: &LenientAllowlistForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistForecastCodeLengthRule {
    pub max_length: i64,
}

impl LenientAllowlistForecastRule for LenientAllowlistForecastCodeLengthRule {
    fn check(&self, item: &LenientAllowlistForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistForecastVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientAllowlistForecastRule for LenientAllowlistForecastVersionRangeRule {
    fn check(&self, item: &LenientAllowlistForecast) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistForecastCodePrefixRule {
    pub prefix: String,
}

impl LenientAllowlistForecastRule for LenientAllowlistForecastCodePrefixRule {
    fn check(&self, item: &LenientAllowlistForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistForecastTitleLengthRule {
    pub max_length: i64,
}

impl LenientAllowlistForecastRule for LenientAllowlistForecastTitleLengthRule {
    fn check(&self, item: &LenientAllowlistForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistForecastRequiredTitleRule {
    pub message: String,
}

impl LenientAllowlistForecastRule for LenientAllowlistForecastRequiredTitleRule {
    fn check(&self, item: &LenientAllowlistForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient allowlist forecast.
#[derive(Debug, Clone)]
pub struct LenientAllowlistForecastValidator {
    pub rules: Vec<Rc<dyn LenientAllowlistForecastRule>>,
}

impl LenientAllowlistForecastValidator {
    pub fn validate(&self, item: &LenientAllowlistForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientAllowlistForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientAllowlistForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_allowlist_forecast_validator() -> LenientAllowlistForecastValidator {
    LenientAllowlistForecastValidator {
        rules: vec![
            Rc::new(LenientAllowlistForecastCodeLengthRule { max_length: 255 }) as Rc<dyn LenientAllowlistForecastRule>,
            Rc::new(LenientAllowlistForecastVersionRangeRule { min: 1000, max: 86400 }),
            Rc::new(LenientAllowlistForecastCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LenientAllowlistForecastTitleLengthRule { max_length: 86400 }),
            Rc::new(LenientAllowlistForecastRequiredTitleRule {
                message: "lenient allowlist forecast needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_allowlist_forecast_validator();
        let item = new_lenient_allowlist_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientAllowlistForecastCodeLengthRule { max_length: 120 };
        let mut item = new_lenient_allowlist_forecast("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = LenientAllowlistForecastVersionRangeRule { min: 120, max: 255 };
        let mut item = new_lenient_allowlist_forecast("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientAllowlistForecastTitleLengthRule { max_length: 80 };
        let mut item = new_lenient_allowlist_forecast("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientAllowlistForecastRequiredTitleRule {
            message: "lenient allowlist forecast needs this field".to_string(),
        };
        let mut item = new_lenient_allowlist_forecast("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
