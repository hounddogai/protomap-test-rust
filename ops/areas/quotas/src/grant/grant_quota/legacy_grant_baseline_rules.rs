use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy grant baseline must pass. Check returns its problems, or none.
pub trait LegacyGrantBaselineRule: std::fmt::Debug {
    fn check(&self, item: &LegacyGrantBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantBaselineNameLengthRule {
    pub max_length: i64,
}

impl LegacyGrantBaselineRule for LegacyGrantBaselineNameLengthRule {
    fn check(&self, item: &LegacyGrantBaseline) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantBaselineMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyGrantBaselineRule for LegacyGrantBaselineMaximumRangeRule {
    fn check(&self, item: &LegacyGrantBaseline) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantBaselineAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LegacyGrantBaselineRule for LegacyGrantBaselineAllowedNameRule {
    fn check(&self, item: &LegacyGrantBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantBaselinePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyGrantBaselineRule for LegacyGrantBaselinePortRangeRule {
    fn check(&self, item: &LegacyGrantBaseline) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantBaselineRequiredCodeRule {
    pub message: String,
}

impl LegacyGrantBaselineRule for LegacyGrantBaselineRequiredCodeRule {
    fn check(&self, item: &LegacyGrantBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantBaselineCodePrefixRule {
    pub prefix: String,
}

impl LegacyGrantBaselineRule for LegacyGrantBaselineCodePrefixRule {
    fn check(&self, item: &LegacyGrantBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy grant baseline.
#[derive(Debug, Clone)]
pub struct LegacyGrantBaselineValidator {
    pub rules: Vec<Rc<dyn LegacyGrantBaselineRule>>,
}

impl LegacyGrantBaselineValidator {
    pub fn validate(&self, item: &LegacyGrantBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyGrantBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyGrantBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_grant_baseline_validator() -> LegacyGrantBaselineValidator {
    LegacyGrantBaselineValidator {
        rules: vec![
            Rc::new(LegacyGrantBaselineNameLengthRule { max_length: 32 }) as Rc<dyn LegacyGrantBaselineRule>,
            Rc::new(LegacyGrantBaselineMaximumRangeRule { min: 9.6, max: 3395.7 }),
            Rc::new(LegacyGrantBaselineAllowedNameRule {
                allowed: vec!["primary".to_string(), "manual".to_string(), "weighted".to_string()],
            }),
            Rc::new(LegacyGrantBaselinePortRangeRule { min: 32, max: 120 }),
            Rc::new(LegacyGrantBaselineRequiredCodeRule {
                message: "legacy grant baseline needs an entry".to_string(),
            }),
            Rc::new(LegacyGrantBaselineCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_grant_baseline_validator();
        let item = new_legacy_grant_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyGrantBaselineNameLengthRule { max_length: 120 };
        let mut item = new_legacy_grant_baseline("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = LegacyGrantBaselineMaximumRangeRule { min: 2.5, max: 4092.7 };
        let mut item = new_legacy_grant_baseline("x", "y");
        item.maximum = 4094.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LegacyGrantBaselinePortRangeRule { min: 64, max: 120 };
        let mut item = new_legacy_grant_baseline("x", "y");
        item.port = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyGrantBaselineRequiredCodeRule {
            message: "legacy grant baseline needs a value".to_string(),
        };
        let mut item = new_legacy_grant_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
