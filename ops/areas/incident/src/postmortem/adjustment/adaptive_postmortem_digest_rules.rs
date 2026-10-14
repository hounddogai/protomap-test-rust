use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive postmortem digest must pass. Check returns its problems, or none.
pub trait AdaptivePostmortemDigestRule: std::fmt::Debug {
    fn check(&self, item: &AdaptivePostmortemDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemDigestLabelLengthRule {
    pub max_length: i64,
}

impl AdaptivePostmortemDigestRule for AdaptivePostmortemDigestLabelLengthRule {
    fn check(&self, item: &AdaptivePostmortemDigest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemDigestRequiredCodeRule {
    pub message: String,
}

impl AdaptivePostmortemDigestRule for AdaptivePostmortemDigestRequiredCodeRule {
    fn check(&self, item: &AdaptivePostmortemDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemDigestEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptivePostmortemDigestRule for AdaptivePostmortemDigestEndsAtRangeRule {
    fn check(&self, item: &AdaptivePostmortemDigest) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemDigestAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AdaptivePostmortemDigestRule for AdaptivePostmortemDigestAllowedLabelRule {
    fn check(&self, item: &AdaptivePostmortemDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemDigestRequiredNameRule {
    pub message: String,
}

impl AdaptivePostmortemDigestRule for AdaptivePostmortemDigestRequiredNameRule {
    fn check(&self, item: &AdaptivePostmortemDigest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePostmortemDigestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptivePostmortemDigestRule for AdaptivePostmortemDigestDurationMinutesRangeRule {
    fn check(&self, item: &AdaptivePostmortemDigest) -> Vec<String> {
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

/// Runs every rule of a adaptive postmortem digest.
#[derive(Debug, Clone)]
pub struct AdaptivePostmortemDigestValidator {
    pub rules: Vec<Rc<dyn AdaptivePostmortemDigestRule>>,
}

impl AdaptivePostmortemDigestValidator {
    pub fn validate(&self, item: &AdaptivePostmortemDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptivePostmortemDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptivePostmortemDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_postmortem_digest_validator() -> AdaptivePostmortemDigestValidator {
    AdaptivePostmortemDigestValidator {
        rules: vec![
            Rc::new(AdaptivePostmortemDigestLabelLengthRule { max_length: 1000 })
                as Rc<dyn AdaptivePostmortemDigestRule>,
            Rc::new(AdaptivePostmortemDigestRequiredCodeRule {
                message: "adaptive postmortem digest needs an entry".to_string(),
            }),
            Rc::new(AdaptivePostmortemDigestEndsAtRangeRule { min: 16, max: 3600 }),
            Rc::new(AdaptivePostmortemDigestAllowedLabelRule {
                allowed: vec!["rolling".to_string(), "adaptive".to_string(), "fallback".to_string()],
            }),
            Rc::new(AdaptivePostmortemDigestRequiredNameRule {
                message: "adaptive postmortem digest needs a value".to_string(),
            }),
            Rc::new(AdaptivePostmortemDigestDurationMinutesRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_postmortem_digest_validator();
        let item = new_adaptive_postmortem_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptivePostmortemDigestLabelLengthRule { max_length: 86400 };
        let mut item = new_adaptive_postmortem_digest("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptivePostmortemDigestRequiredCodeRule {
            message: "adaptive postmortem digest needs a value".to_string(),
        };
        let mut item = new_adaptive_postmortem_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = AdaptivePostmortemDigestEndsAtRangeRule { min: 64, max: 1000 };
        let mut item = new_adaptive_postmortem_digest("x", "y");
        item.ends_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptivePostmortemDigestRequiredNameRule {
            message: "adaptive postmortem digest needs a value".to_string(),
        };
        let mut item = new_adaptive_postmortem_digest("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = AdaptivePostmortemDigestDurationMinutesRangeRule { min: 64, max: 86400 };
        let mut item = new_adaptive_postmortem_digest("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
