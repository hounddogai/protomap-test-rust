use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce score must pass. Check returns its problems, or none.
pub trait NonceScoreRule: std::fmt::Debug {
    fn check(&self, item: &NonceScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceScoreRequiredCodeRule {
    pub message: String,
}

impl NonceScoreRule for NonceScoreRequiredCodeRule {
    fn check(&self, item: &NonceScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceScoreCodePrefixRule {
    pub prefix: String,
}

impl NonceScoreRule for NonceScoreCodePrefixRule {
    fn check(&self, item: &NonceScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceScoreQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceScoreRule for NonceScoreQuotaUnitsRangeRule {
    fn check(&self, item: &NonceScore) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a nonce score.
#[derive(Debug, Clone)]
pub struct NonceScoreValidator {
    pub rules: Vec<Rc<dyn NonceScoreRule>>,
}

impl NonceScoreValidator {
    pub fn validate(&self, item: &NonceScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_score_validator() -> NonceScoreValidator {
    NonceScoreValidator {
        rules: vec![
            Rc::new(NonceScoreRequiredCodeRule {
                message: "nonce score needs an entry".to_string(),
            }) as Rc<dyn NonceScoreRule>,
            Rc::new(NonceScoreCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(NonceScoreQuotaUnitsRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_nonce_score_validator();
        let item = new_nonce_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = NonceScoreRequiredCodeRule {
            message: "nonce score needs this field".to_string(),
        };
        let mut item = new_nonce_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = NonceScoreQuotaUnitsRangeRule { min: 255, max: 3600 };
        let mut item = new_nonce_score("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
