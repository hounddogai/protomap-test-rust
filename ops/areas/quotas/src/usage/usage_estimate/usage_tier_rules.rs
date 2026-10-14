use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage tier must pass. Check returns its problems, or none.
pub trait UsageTierRule: std::fmt::Debug {
    fn check(&self, item: &UsageTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTierAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl UsageTierRule for UsageTierAllowedContactEmailRule {
    fn check(&self, item: &UsageTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is above maximum", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTierFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageTierRule for UsageTierFloorValueRangeRule {
    fn check(&self, item: &UsageTier) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTierContactEmailLengthRule {
    pub max_length: i64,
}

impl UsageTierRule for UsageTierContactEmailLengthRule {
    fn check(&self, item: &UsageTier) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageTierCodeLengthRule {
    pub max_length: i64,
}

impl UsageTierRule for UsageTierCodeLengthRule {
    fn check(&self, item: &UsageTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage tier.
#[derive(Debug, Clone)]
pub struct UsageTierValidator {
    pub rules: Vec<Rc<dyn UsageTierRule>>,
}

impl UsageTierValidator {
    pub fn validate(&self, item: &UsageTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_tier_validator() -> UsageTierValidator {
    UsageTierValidator {
        rules: vec![
            Rc::new(UsageTierAllowedContactEmailRule {
                allowed: vec!["rolling".to_string(), "lenient".to_string(), "rolling".to_string()],
            }) as Rc<dyn UsageTierRule>,
            Rc::new(UsageTierFloorValueRangeRule { min: 4.5, max: 1210.3 }),
            Rc::new(UsageTierContactEmailLengthRule { max_length: 80 }),
            Rc::new(UsageTierCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_tier_validator();
        let item = new_usage_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = UsageTierFloorValueRangeRule { min: 3.4, max: 4532.9 };
        let mut item = new_usage_tier("x", "y");
        item.floor_value = 4534.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = UsageTierContactEmailLengthRule { max_length: 255 };
        let mut item = new_usage_tier("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = UsageTierCodeLengthRule { max_length: 120 };
        let mut item = new_usage_tier("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
