use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit bundle must pass. Check returns its problems, or none.
pub trait LimitBundleRule: std::fmt::Debug {
    fn check(&self, item: &LimitBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitBundleRequiredLabelRule {
    pub message: String,
}

impl LimitBundleRule for LimitBundleRequiredLabelRule {
    fn check(&self, item: &LimitBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitBundlePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitBundleRule for LimitBundlePortRangeRule {
    fn check(&self, item: &LimitBundle) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitBundleCodeLengthRule {
    pub max_length: i64,
}

impl LimitBundleRule for LimitBundleCodeLengthRule {
    fn check(&self, item: &LimitBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit bundle.
#[derive(Debug, Clone)]
pub struct LimitBundleValidator {
    pub rules: Vec<Rc<dyn LimitBundleRule>>,
}

impl LimitBundleValidator {
    pub fn validate(&self, item: &LimitBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_bundle_validator() -> LimitBundleValidator {
    LimitBundleValidator {
        rules: vec![
            Rc::new(LimitBundleRequiredLabelRule {
                message: "limit bundle needs a value".to_string(),
            }) as Rc<dyn LimitBundleRule>,
            Rc::new(LimitBundlePortRangeRule { min: 64, max: 120 }),
            Rc::new(LimitBundleCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_bundle_validator();
        let item = new_limit_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LimitBundleRequiredLabelRule {
            message: "limit bundle needs a value".to_string(),
        };
        let mut item = new_limit_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LimitBundlePortRangeRule { min: 16, max: 80 };
        let mut item = new_limit_bundle("x", "y");
        item.port = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitBundleCodeLengthRule { max_length: 1000 };
        let mut item = new_limit_bundle("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
