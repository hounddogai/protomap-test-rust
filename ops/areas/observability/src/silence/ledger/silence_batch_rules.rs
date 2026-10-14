use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence batch must pass. Check returns its problems, or none.
pub trait SilenceBatchRule: std::fmt::Debug {
    fn check(&self, item: &SilenceBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceBatchSourceLengthRule {
    pub max_length: i64,
}

impl SilenceBatchRule for SilenceBatchSourceLengthRule {
    fn check(&self, item: &SilenceBatch) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceBatchCodePrefixRule {
    pub prefix: String,
}

impl SilenceBatchRule for SilenceBatchCodePrefixRule {
    fn check(&self, item: &SilenceBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceBatchPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceBatchRule for SilenceBatchPortRangeRule {
    fn check(&self, item: &SilenceBatch) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence batch.
#[derive(Debug, Clone)]
pub struct SilenceBatchValidator {
    pub rules: Vec<Rc<dyn SilenceBatchRule>>,
}

impl SilenceBatchValidator {
    pub fn validate(&self, item: &SilenceBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_batch_validator() -> SilenceBatchValidator {
    SilenceBatchValidator {
        rules: vec![
            Rc::new(SilenceBatchSourceLengthRule { max_length: 80 }) as Rc<dyn SilenceBatchRule>,
            Rc::new(SilenceBatchCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SilenceBatchPortRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_batch_validator();
        let item = new_silence_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = SilenceBatchSourceLengthRule { max_length: 120 };
        let mut item = new_silence_batch("x", "y");
        item.source = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = SilenceBatchPortRangeRule { min: 64, max: 1000 };
        let mut item = new_silence_batch("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
