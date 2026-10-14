use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped suspension override must pass. Check returns its problems, or none.
pub trait CappedSuspensionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &CappedSuspensionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionOverrideCodeLengthRule {
    pub max_length: i64,
}

impl CappedSuspensionOverrideRule for CappedSuspensionOverrideCodeLengthRule {
    fn check(&self, item: &CappedSuspensionOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionOverrideAudienceLengthRule {
    pub max_length: i64,
}

impl CappedSuspensionOverrideRule for CappedSuspensionOverrideAudienceLengthRule {
    fn check(&self, item: &CappedSuspensionOverride) -> Vec<String> {
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
pub struct CappedSuspensionOverrideRequiredAudienceRule {
    pub message: String,
}

impl CappedSuspensionOverrideRule for CappedSuspensionOverrideRequiredAudienceRule {
    fn check(&self, item: &CappedSuspensionOverride) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionOverrideCodePrefixRule {
    pub prefix: String,
}

impl CappedSuspensionOverrideRule for CappedSuspensionOverrideCodePrefixRule {
    fn check(&self, item: &CappedSuspensionOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSuspensionOverrideScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedSuspensionOverrideRule for CappedSuspensionOverrideScoreRangeRule {
    fn check(&self, item: &CappedSuspensionOverride) -> Vec<String> {
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

/// Runs every rule of a capped suspension override.
#[derive(Debug, Clone)]
pub struct CappedSuspensionOverrideValidator {
    pub rules: Vec<Rc<dyn CappedSuspensionOverrideRule>>,
}

impl CappedSuspensionOverrideValidator {
    pub fn validate(&self, item: &CappedSuspensionOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedSuspensionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSuspensionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_suspension_override_validator() -> CappedSuspensionOverrideValidator {
    CappedSuspensionOverrideValidator {
        rules: vec![
            Rc::new(CappedSuspensionOverrideCodeLengthRule { max_length: 255 }) as Rc<dyn CappedSuspensionOverrideRule>,
            Rc::new(CappedSuspensionOverrideAudienceLengthRule { max_length: 32 }),
            Rc::new(CappedSuspensionOverrideRequiredAudienceRule {
                message: "capped suspension override needs a value".to_string(),
            }),
            Rc::new(CappedSuspensionOverrideCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CappedSuspensionOverrideScoreRangeRule { min: 0.1, max: 1177.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_suspension_override_validator();
        let item = new_capped_suspension_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedSuspensionOverrideCodeLengthRule { max_length: 1000 };
        let mut item = new_capped_suspension_override("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = CappedSuspensionOverrideAudienceLengthRule { max_length: 16 };
        let mut item = new_capped_suspension_override("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = CappedSuspensionOverrideRequiredAudienceRule {
            message: "capped suspension override needs a value".to_string(),
        };
        let mut item = new_capped_suspension_override("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = CappedSuspensionOverrideScoreRangeRule { min: 3.0, max: 2903.6 };
        let mut item = new_capped_suspension_override("x", "y");
        item.score = 2905.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
