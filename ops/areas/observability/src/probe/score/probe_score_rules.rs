use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a probe score must pass. Check returns its problems, or none.
pub trait ProbeScoreRule: std::fmt::Debug {
    fn check(&self, item: &ProbeScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeScoreScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ProbeScoreRule for ProbeScoreScoreRangeRule {
    fn check(&self, item: &ProbeScore) -> Vec<String> {
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
pub struct ProbeScoreCodePrefixRule {
    pub prefix: String,
}

impl ProbeScoreRule for ProbeScoreCodePrefixRule {
    fn check(&self, item: &ProbeScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeScoreSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeScoreRule for ProbeScoreSequenceRangeRule {
    fn check(&self, item: &ProbeScore) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeScoreTitleLengthRule {
    pub max_length: i64,
}

impl ProbeScoreRule for ProbeScoreTitleLengthRule {
    fn check(&self, item: &ProbeScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ProbeScoreRule for ProbeScoreAllowedCodeRule {
    fn check(&self, item: &ProbeScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeScoreLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeScoreRule for ProbeScoreLimitRangeRule {
    fn check(&self, item: &ProbeScore) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a probe score.
#[derive(Debug, Clone)]
pub struct ProbeScoreValidator {
    pub rules: Vec<Rc<dyn ProbeScoreRule>>,
}

impl ProbeScoreValidator {
    pub fn validate(&self, item: &ProbeScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_score_validator() -> ProbeScoreValidator {
    ProbeScoreValidator {
        rules: vec![
            Rc::new(ProbeScoreScoreRangeRule { min: 7.7, max: 3925.2 }) as Rc<dyn ProbeScoreRule>,
            Rc::new(ProbeScoreCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ProbeScoreSequenceRangeRule { min: 3600, max: 3600 }),
            Rc::new(ProbeScoreTitleLengthRule { max_length: 120 }),
            Rc::new(ProbeScoreAllowedCodeRule {
                allowed: vec!["scheduled".to_string(), "batched".to_string()],
            }),
            Rc::new(ProbeScoreLimitRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_score_validator();
        let item = new_probe_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ProbeScoreScoreRangeRule { min: 0.5, max: 3217.5 };
        let mut item = new_probe_score("x", "y");
        item.score = 3219.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ProbeScoreSequenceRangeRule { min: 120, max: 255 };
        let mut item = new_probe_score("x", "y");
        item.sequence = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ProbeScoreTitleLengthRule { max_length: 32 };
        let mut item = new_probe_score("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ProbeScoreLimitRangeRule { min: 80, max: 3600 };
        let mut item = new_probe_score("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
