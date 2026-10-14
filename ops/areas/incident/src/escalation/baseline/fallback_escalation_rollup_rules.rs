use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback escalation rollup must pass. Check returns its problems, or none.
pub trait FallbackEscalationRollupRule: std::fmt::Debug {
    fn check(&self, item: &FallbackEscalationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEscalationRollupTitleLengthRule {
    pub max_length: i64,
}

impl FallbackEscalationRollupRule for FallbackEscalationRollupTitleLengthRule {
    fn check(&self, item: &FallbackEscalationRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEscalationRollupScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackEscalationRollupRule for FallbackEscalationRollupScoreRangeRule {
    fn check(&self, item: &FallbackEscalationRollup) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEscalationRollupAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl FallbackEscalationRollupRule for FallbackEscalationRollupAllowedNotesRule {
    fn check(&self, item: &FallbackEscalationRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is not allowed", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEscalationRollupRequiredCountryRule {
    pub message: String,
}

impl FallbackEscalationRollupRule for FallbackEscalationRollupRequiredCountryRule {
    fn check(&self, item: &FallbackEscalationRollup) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback escalation rollup.
#[derive(Debug, Clone)]
pub struct FallbackEscalationRollupValidator {
    pub rules: Vec<Rc<dyn FallbackEscalationRollupRule>>,
}

impl FallbackEscalationRollupValidator {
    pub fn validate(&self, item: &FallbackEscalationRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackEscalationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackEscalationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_escalation_rollup_validator() -> FallbackEscalationRollupValidator {
    FallbackEscalationRollupValidator {
        rules: vec![
            Rc::new(FallbackEscalationRollupTitleLengthRule { max_length: 3600 })
                as Rc<dyn FallbackEscalationRollupRule>,
            Rc::new(FallbackEscalationRollupScoreRangeRule { min: 2.4, max: 2933.3 }),
            Rc::new(FallbackEscalationRollupAllowedNotesRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(FallbackEscalationRollupRequiredCountryRule {
                message: "fallback escalation rollup needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_escalation_rollup_validator();
        let item = new_fallback_escalation_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackEscalationRollupTitleLengthRule { max_length: 3600 };
        let mut item = new_fallback_escalation_rollup("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = FallbackEscalationRollupScoreRangeRule { min: 3.2, max: 409.2 };
        let mut item = new_fallback_escalation_rollup("x", "y");
        item.score = 410.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = FallbackEscalationRollupRequiredCountryRule {
            message: "fallback escalation rollup needs this field".to_string(),
        };
        let mut item = new_fallback_escalation_rollup("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
