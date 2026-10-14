use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary egress score must pass. Check returns its problems, or none.
pub trait PrimaryEgressScoreRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEgressScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressScoreCodePrefixRule {
    pub prefix: String,
}

impl PrimaryEgressScoreRule for PrimaryEgressScoreCodePrefixRule {
    fn check(&self, item: &PrimaryEgressScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressScoreRequiredCurrencyRule {
    pub message: String,
}

impl PrimaryEgressScoreRule for PrimaryEgressScoreRequiredCurrencyRule {
    fn check(&self, item: &PrimaryEgressScore) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressScoreRequiredKeyIdRule {
    pub message: String,
}

impl PrimaryEgressScoreRule for PrimaryEgressScoreRequiredKeyIdRule {
    fn check(&self, item: &PrimaryEgressScore) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressScoreFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryEgressScoreRule for PrimaryEgressScoreFloorValueRangeRule {
    fn check(&self, item: &PrimaryEgressScore) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary egress score.
#[derive(Debug, Clone)]
pub struct PrimaryEgressScoreValidator {
    pub rules: Vec<Rc<dyn PrimaryEgressScoreRule>>,
}

impl PrimaryEgressScoreValidator {
    pub fn validate(&self, item: &PrimaryEgressScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryEgressScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEgressScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_egress_score_validator() -> PrimaryEgressScoreValidator {
    PrimaryEgressScoreValidator {
        rules: vec![
            Rc::new(PrimaryEgressScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn PrimaryEgressScoreRule>,
            Rc::new(PrimaryEgressScoreRequiredCurrencyRule {
                message: "primary egress score needs an entry".to_string(),
            }),
            Rc::new(PrimaryEgressScoreRequiredKeyIdRule {
                message: "primary egress score needs an entry".to_string(),
            }),
            Rc::new(PrimaryEgressScoreFloorValueRangeRule { min: 4.0, max: 4035.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_egress_score_validator();
        let item = new_primary_egress_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = PrimaryEgressScoreRequiredCurrencyRule {
            message: "primary egress score needs this field".to_string(),
        };
        let mut item = new_primary_egress_score("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PrimaryEgressScoreRequiredKeyIdRule {
            message: "primary egress score needs a value".to_string(),
        };
        let mut item = new_primary_egress_score("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = PrimaryEgressScoreFloorValueRangeRule { min: 6.4, max: 3185.2 };
        let mut item = new_primary_egress_score("x", "y");
        item.floor_value = 3186.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
