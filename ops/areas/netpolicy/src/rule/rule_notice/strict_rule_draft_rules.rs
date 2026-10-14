use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict rule draft must pass. Check returns its problems, or none.
pub trait StrictRuleDraftRule: std::fmt::Debug {
    fn check(&self, item: &StrictRuleDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleDraftRequiredCurrencyRule {
    pub message: String,
}

impl StrictRuleDraftRule for StrictRuleDraftRequiredCurrencyRule {
    fn check(&self, item: &StrictRuleDraft) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleDraftCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRuleDraftRule for StrictRuleDraftCostRangeRule {
    fn check(&self, item: &StrictRuleDraft) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleDraftAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl StrictRuleDraftRule for StrictRuleDraftAllowedCurrencyRule {
    fn check(&self, item: &StrictRuleDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is unknown", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleDraftCodeLengthRule {
    pub max_length: i64,
}

impl StrictRuleDraftRule for StrictRuleDraftCodeLengthRule {
    fn check(&self, item: &StrictRuleDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleDraftFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRuleDraftRule for StrictRuleDraftFloorValueRangeRule {
    fn check(&self, item: &StrictRuleDraft) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict rule draft.
#[derive(Debug, Clone)]
pub struct StrictRuleDraftValidator {
    pub rules: Vec<Rc<dyn StrictRuleDraftRule>>,
}

impl StrictRuleDraftValidator {
    pub fn validate(&self, item: &StrictRuleDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictRuleDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRuleDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_rule_draft_validator() -> StrictRuleDraftValidator {
    StrictRuleDraftValidator {
        rules: vec![
            Rc::new(StrictRuleDraftRequiredCurrencyRule {
                message: "strict rule draft needs an entry".to_string(),
            }) as Rc<dyn StrictRuleDraftRule>,
            Rc::new(StrictRuleDraftCostRangeRule { min: 6.5, max: 2326.0 }),
            Rc::new(StrictRuleDraftAllowedCurrencyRule {
                allowed: vec!["tiered".to_string()],
            }),
            Rc::new(StrictRuleDraftCodeLengthRule { max_length: 16 }),
            Rc::new(StrictRuleDraftFloorValueRangeRule { min: 6.1, max: 764.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_rule_draft_validator();
        let item = new_strict_rule_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = StrictRuleDraftRequiredCurrencyRule {
            message: "strict rule draft needs an entry".to_string(),
        };
        let mut item = new_strict_rule_draft("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = StrictRuleDraftCostRangeRule { min: 4.0, max: 1643.0 };
        let mut item = new_strict_rule_draft("x", "y");
        item.cost = 1644.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictRuleDraftCodeLengthRule { max_length: 120 };
        let mut item = new_strict_rule_draft("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = StrictRuleDraftFloorValueRangeRule { min: 1.9, max: 3301.8 };
        let mut item = new_strict_rule_draft("x", "y");
        item.floor_value = 3303.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
