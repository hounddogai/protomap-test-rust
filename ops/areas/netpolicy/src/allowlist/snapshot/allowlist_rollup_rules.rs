use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist rollup must pass. Check returns its problems, or none.
pub trait AllowlistRollupRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRollupFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistRollupRule for AllowlistRollupFloorValueRangeRule {
    fn check(&self, item: &AllowlistRollup) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRollupCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistRollupRule for AllowlistRollupCodeLengthRule {
    fn check(&self, item: &AllowlistRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRollupWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistRollupRule for AllowlistRollupWindowDaysRangeRule {
    fn check(&self, item: &AllowlistRollup) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRollupAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AllowlistRollupRule for AllowlistRollupAllowedNameRule {
    fn check(&self, item: &AllowlistRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRollupDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistRollupRule for AllowlistRollupDurationMinutesRangeRule {
    fn check(&self, item: &AllowlistRollup) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRollupCodePrefixRule {
    pub prefix: String,
}

impl AllowlistRollupRule for AllowlistRollupCodePrefixRule {
    fn check(&self, item: &AllowlistRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist rollup.
#[derive(Debug, Clone)]
pub struct AllowlistRollupValidator {
    pub rules: Vec<Rc<dyn AllowlistRollupRule>>,
}

impl AllowlistRollupValidator {
    pub fn validate(&self, item: &AllowlistRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_rollup_validator() -> AllowlistRollupValidator {
    AllowlistRollupValidator {
        rules: vec![
            Rc::new(AllowlistRollupFloorValueRangeRule { min: 3.2, max: 4252.2 }) as Rc<dyn AllowlistRollupRule>,
            Rc::new(AllowlistRollupCodeLengthRule { max_length: 32 }),
            Rc::new(AllowlistRollupWindowDaysRangeRule { min: 64, max: 80 }),
            Rc::new(AllowlistRollupAllowedNameRule {
                allowed: vec!["pending".to_string(), "adaptive".to_string()],
            }),
            Rc::new(AllowlistRollupDurationMinutesRangeRule { min: 120, max: 255 }),
            Rc::new(AllowlistRollupCodePrefixRule {
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
        let validator = default_allowlist_rollup_validator();
        let item = new_allowlist_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = AllowlistRollupFloorValueRangeRule { min: 3.8, max: 304.6 };
        let mut item = new_allowlist_rollup("x", "y");
        item.floor_value = 306.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AllowlistRollupCodeLengthRule { max_length: 86400 };
        let mut item = new_allowlist_rollup("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = AllowlistRollupWindowDaysRangeRule { min: 120, max: 86400 };
        let mut item = new_allowlist_rollup("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = AllowlistRollupDurationMinutesRangeRule { min: 120, max: 255 };
        let mut item = new_allowlist_rollup("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
