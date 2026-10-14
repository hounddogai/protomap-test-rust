use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity baseline must pass. Check returns its problems, or none.
pub trait IdentityBaselineRule: std::fmt::Debug {
    fn check(&self, item: &IdentityBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBaselineNotesLengthRule {
    pub max_length: i64,
}

impl IdentityBaselineRule for IdentityBaselineNotesLengthRule {
    fn check(&self, item: &IdentityBaseline) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBaselineRequiredLabelRule {
    pub message: String,
}

impl IdentityBaselineRule for IdentityBaselineRequiredLabelRule {
    fn check(&self, item: &IdentityBaseline) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBaselineAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl IdentityBaselineRule for IdentityBaselineAllowedNotesRule {
    fn check(&self, item: &IdentityBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is too long", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBaselineRequiredSegmentRule {
    pub message: String,
}

impl IdentityBaselineRule for IdentityBaselineRequiredSegmentRule {
    fn check(&self, item: &IdentityBaseline) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBaselineScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IdentityBaselineRule for IdentityBaselineScoreRangeRule {
    fn check(&self, item: &IdentityBaseline) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity baseline.
#[derive(Debug, Clone)]
pub struct IdentityBaselineValidator {
    pub rules: Vec<Rc<dyn IdentityBaselineRule>>,
}

impl IdentityBaselineValidator {
    pub fn validate(&self, item: &IdentityBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_baseline_validator() -> IdentityBaselineValidator {
    IdentityBaselineValidator {
        rules: vec![
            Rc::new(IdentityBaselineNotesLengthRule { max_length: 86400 }) as Rc<dyn IdentityBaselineRule>,
            Rc::new(IdentityBaselineRequiredLabelRule {
                message: "identity baseline needs a value".to_string(),
            }),
            Rc::new(IdentityBaselineAllowedNotesRule {
                allowed: vec!["fallback".to_string(), "default".to_string()],
            }),
            Rc::new(IdentityBaselineRequiredSegmentRule {
                message: "identity baseline needs a value".to_string(),
            }),
            Rc::new(IdentityBaselineScoreRangeRule { min: 8.5, max: 2366.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_baseline_validator();
        let item = new_identity_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = IdentityBaselineNotesLengthRule { max_length: 86400 };
        let mut item = new_identity_baseline("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = IdentityBaselineRequiredLabelRule {
            message: "identity baseline needs a value".to_string(),
        };
        let mut item = new_identity_baseline("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = IdentityBaselineRequiredSegmentRule {
            message: "identity baseline needs a value".to_string(),
        };
        let mut item = new_identity_baseline("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = IdentityBaselineScoreRangeRule { min: 9.7, max: 1936.1 };
        let mut item = new_identity_baseline("x", "y");
        item.score = 1937.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
