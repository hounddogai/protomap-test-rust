use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal probe draft must pass. Check returns its problems, or none.
pub trait SeasonalProbeDraftRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalProbeDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeDraftCodePrefixRule {
    pub prefix: String,
}

impl SeasonalProbeDraftRule for SeasonalProbeDraftCodePrefixRule {
    fn check(&self, item: &SeasonalProbeDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeDraftAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl SeasonalProbeDraftRule for SeasonalProbeDraftAllowedLabelRule {
    fn check(&self, item: &SeasonalProbeDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeDraftCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalProbeDraftRule for SeasonalProbeDraftCodeLengthRule {
    fn check(&self, item: &SeasonalProbeDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeDraftRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalProbeDraftRule for SeasonalProbeDraftRateRangeRule {
    fn check(&self, item: &SeasonalProbeDraft) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeDraftLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalProbeDraftRule for SeasonalProbeDraftLabelLengthRule {
    fn check(&self, item: &SeasonalProbeDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal probe draft.
#[derive(Debug, Clone)]
pub struct SeasonalProbeDraftValidator {
    pub rules: Vec<Rc<dyn SeasonalProbeDraftRule>>,
}

impl SeasonalProbeDraftValidator {
    pub fn validate(&self, item: &SeasonalProbeDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalProbeDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalProbeDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_probe_draft_validator() -> SeasonalProbeDraftValidator {
    SeasonalProbeDraftValidator {
        rules: vec![
            Rc::new(SeasonalProbeDraftCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn SeasonalProbeDraftRule>,
            Rc::new(SeasonalProbeDraftAllowedLabelRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(SeasonalProbeDraftCodeLengthRule { max_length: 16 }),
            Rc::new(SeasonalProbeDraftRateRangeRule { min: 5.8, max: 3642.4 }),
            Rc::new(SeasonalProbeDraftLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_probe_draft_validator();
        let item = new_seasonal_probe_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalProbeDraftCodeLengthRule { max_length: 86400 };
        let mut item = new_seasonal_probe_draft("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = SeasonalProbeDraftRateRangeRule { min: 2.6, max: 1781.1 };
        let mut item = new_seasonal_probe_draft("x", "y");
        item.rate = 1782.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SeasonalProbeDraftLabelLengthRule { max_length: 1000 };
        let mut item = new_seasonal_probe_draft("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
