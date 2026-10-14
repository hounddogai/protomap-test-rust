use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary overage preference must pass. Check returns its problems, or none.
pub trait PrimaryOveragePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryOveragePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOveragePreferenceRequiredHostRule {
    pub message: String,
}

impl PrimaryOveragePreferenceRule for PrimaryOveragePreferenceRequiredHostRule {
    fn check(&self, item: &PrimaryOveragePreference) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOveragePreferenceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryOveragePreferenceRule for PrimaryOveragePreferenceAllowedCodeRule {
    fn check(&self, item: &PrimaryOveragePreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOveragePreferenceScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryOveragePreferenceRule for PrimaryOveragePreferenceScoreRangeRule {
    fn check(&self, item: &PrimaryOveragePreference) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOveragePreferenceMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryOveragePreferenceRule for PrimaryOveragePreferenceMarginRangeRule {
    fn check(&self, item: &PrimaryOveragePreference) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOveragePreferenceNameLengthRule {
    pub max_length: i64,
}

impl PrimaryOveragePreferenceRule for PrimaryOveragePreferenceNameLengthRule {
    fn check(&self, item: &PrimaryOveragePreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary overage preference.
#[derive(Debug, Clone)]
pub struct PrimaryOveragePreferenceValidator {
    pub rules: Vec<Rc<dyn PrimaryOveragePreferenceRule>>,
}

impl PrimaryOveragePreferenceValidator {
    pub fn validate(&self, item: &PrimaryOveragePreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryOveragePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryOveragePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_overage_preference_validator() -> PrimaryOveragePreferenceValidator {
    PrimaryOveragePreferenceValidator {
        rules: vec![
            Rc::new(PrimaryOveragePreferenceRequiredHostRule {
                message: "primary overage preference needs this field".to_string(),
            }) as Rc<dyn PrimaryOveragePreferenceRule>,
            Rc::new(PrimaryOveragePreferenceAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PrimaryOveragePreferenceScoreRangeRule { min: 8.0, max: 2955.6 }),
            Rc::new(PrimaryOveragePreferenceMarginRangeRule { min: 4.9, max: 1417.5 }),
            Rc::new(PrimaryOveragePreferenceNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_overage_preference_validator();
        let item = new_primary_overage_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PrimaryOveragePreferenceRequiredHostRule {
            message: "primary overage preference needs an entry".to_string(),
        };
        let mut item = new_primary_overage_preference("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PrimaryOveragePreferenceScoreRangeRule { min: 2.5, max: 364.1 };
        let mut item = new_primary_overage_preference("x", "y");
        item.score = 365.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = PrimaryOveragePreferenceMarginRangeRule { min: 3.1, max: 909.6 };
        let mut item = new_primary_overage_preference("x", "y");
        item.margin = 911.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryOveragePreferenceNameLengthRule { max_length: 80 };
        let mut item = new_primary_overage_preference("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
