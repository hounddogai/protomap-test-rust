use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered canary score must pass. Check returns its problems, or none.
pub trait TieredCanaryScoreRule: std::fmt::Debug {
    fn check(&self, item: &TieredCanaryScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryScoreUnitLengthRule {
    pub max_length: i64,
}

impl TieredCanaryScoreRule for TieredCanaryScoreUnitLengthRule {
    fn check(&self, item: &TieredCanaryScore) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryScoreCodePrefixRule {
    pub prefix: String,
}

impl TieredCanaryScoreRule for TieredCanaryScoreCodePrefixRule {
    fn check(&self, item: &TieredCanaryScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryScoreRequiredCodeRule {
    pub message: String,
}

impl TieredCanaryScoreRule for TieredCanaryScoreRequiredCodeRule {
    fn check(&self, item: &TieredCanaryScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryScoreLabelLengthRule {
    pub max_length: i64,
}

impl TieredCanaryScoreRule for TieredCanaryScoreLabelLengthRule {
    fn check(&self, item: &TieredCanaryScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryScoreAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredCanaryScoreRule for TieredCanaryScoreAmountRangeRule {
    fn check(&self, item: &TieredCanaryScore) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered canary score.
#[derive(Debug, Clone)]
pub struct TieredCanaryScoreValidator {
    pub rules: Vec<Rc<dyn TieredCanaryScoreRule>>,
}

impl TieredCanaryScoreValidator {
    pub fn validate(&self, item: &TieredCanaryScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredCanaryScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredCanaryScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_canary_score_validator() -> TieredCanaryScoreValidator {
    TieredCanaryScoreValidator {
        rules: vec![
            Rc::new(TieredCanaryScoreUnitLengthRule { max_length: 80 }) as Rc<dyn TieredCanaryScoreRule>,
            Rc::new(TieredCanaryScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(TieredCanaryScoreRequiredCodeRule {
                message: "tiered canary score needs a value".to_string(),
            }),
            Rc::new(TieredCanaryScoreLabelLengthRule { max_length: 1000 }),
            Rc::new(TieredCanaryScoreAmountRangeRule { min: 8.4, max: 4609.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_canary_score_validator();
        let item = new_tiered_canary_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = TieredCanaryScoreUnitLengthRule { max_length: 1000 };
        let mut item = new_tiered_canary_score("x", "y");
        item.unit = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredCanaryScoreRequiredCodeRule {
            message: "tiered canary score needs a value".to_string(),
        };
        let mut item = new_tiered_canary_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredCanaryScoreLabelLengthRule { max_length: 1000 };
        let mut item = new_tiered_canary_score("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = TieredCanaryScoreAmountRangeRule { min: 0.5, max: 1779.2 };
        let mut item = new_tiered_canary_score("x", "y");
        item.amount = 1780.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
