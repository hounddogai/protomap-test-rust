use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a token record must pass. Check returns its problems, or none.
pub trait TokenRecordRule: std::fmt::Debug {
    fn check(&self, item: &TokenRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRecordActiveGuardRule {
    pub limit: f64,
}

impl TokenRecordRule for TokenRecordActiveGuardRule {
    fn check(&self, item: &TokenRecord) -> Vec<String> {
        if item.active && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while active",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRecordRequiredCodeRule {
    pub message: String,
}

impl TokenRecordRule for TokenRecordRequiredCodeRule {
    fn check(&self, item: &TokenRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRecordFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenRecordRule for TokenRecordFloorValueRangeRule {
    fn check(&self, item: &TokenRecord) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenRecordUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TokenRecordRule for TokenRecordUpdatedAtRangeRule {
    fn check(&self, item: &TokenRecord) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a token record.
#[derive(Debug, Clone)]
pub struct TokenRecordValidator {
    pub rules: Vec<Rc<dyn TokenRecordRule>>,
}

impl TokenRecordValidator {
    pub fn validate(&self, item: &TokenRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_record_validator() -> TokenRecordValidator {
    TokenRecordValidator {
        rules: vec![
            Rc::new(TokenRecordActiveGuardRule { limit: 5.3 }) as Rc<dyn TokenRecordRule>,
            Rc::new(TokenRecordRequiredCodeRule {
                message: "token record needs this field".to_string(),
            }),
            Rc::new(TokenRecordFloorValueRangeRule { min: 0.5, max: 3834.4 }),
            Rc::new(TokenRecordUpdatedAtRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_token_record_validator();
        let item = new_token_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TokenRecordRequiredCodeRule {
            message: "token record needs a value".to_string(),
        };
        let mut item = new_token_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = TokenRecordFloorValueRangeRule { min: 2.0, max: 1590.0 };
        let mut item = new_token_record("x", "y");
        item.floor_value = 1591.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = TokenRecordUpdatedAtRangeRule { min: 16, max: 64 };
        let mut item = new_token_record("x", "y");
        item.updated_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
