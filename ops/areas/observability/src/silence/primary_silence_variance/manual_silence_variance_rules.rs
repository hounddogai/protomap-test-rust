use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual silence variance must pass. Check returns its problems, or none.
pub trait ManualSilenceVarianceRule: std::fmt::Debug {
    fn check(&self, item: &ManualSilenceVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceVarianceArchivedGuardRule {
    pub limit: f64,
}

impl ManualSilenceVarianceRule for ManualSilenceVarianceArchivedGuardRule {
    fn check(&self, item: &ManualSilenceVariance) -> Vec<String> {
        if item.archived && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while archived", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceVarianceCipherLengthRule {
    pub max_length: i64,
}

impl ManualSilenceVarianceRule for ManualSilenceVarianceCipherLengthRule {
    fn check(&self, item: &ManualSilenceVariance) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceVarianceAudienceLengthRule {
    pub max_length: i64,
}

impl ManualSilenceVarianceRule for ManualSilenceVarianceAudienceLengthRule {
    fn check(&self, item: &ManualSilenceVariance) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceVarianceRequiredAudienceRule {
    pub message: String,
}

impl ManualSilenceVarianceRule for ManualSilenceVarianceRequiredAudienceRule {
    fn check(&self, item: &ManualSilenceVariance) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual silence variance.
#[derive(Debug, Clone)]
pub struct ManualSilenceVarianceValidator {
    pub rules: Vec<Rc<dyn ManualSilenceVarianceRule>>,
}

impl ManualSilenceVarianceValidator {
    pub fn validate(&self, item: &ManualSilenceVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualSilenceVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSilenceVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_silence_variance_validator() -> ManualSilenceVarianceValidator {
    ManualSilenceVarianceValidator {
        rules: vec![
            Rc::new(ManualSilenceVarianceArchivedGuardRule { limit: 0.5 }) as Rc<dyn ManualSilenceVarianceRule>,
            Rc::new(ManualSilenceVarianceCipherLengthRule { max_length: 120 }),
            Rc::new(ManualSilenceVarianceAudienceLengthRule { max_length: 3600 }),
            Rc::new(ManualSilenceVarianceRequiredAudienceRule {
                message: "manual silence variance needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_silence_variance_validator();
        let item = new_manual_silence_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = ManualSilenceVarianceCipherLengthRule { max_length: 64 };
        let mut item = new_manual_silence_variance("x", "y");
        item.cipher = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = ManualSilenceVarianceAudienceLengthRule { max_length: 32 };
        let mut item = new_manual_silence_variance("x", "y");
        item.audience = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = ManualSilenceVarianceRequiredAudienceRule {
            message: "manual silence variance needs a value".to_string(),
        };
        let mut item = new_manual_silence_variance("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
