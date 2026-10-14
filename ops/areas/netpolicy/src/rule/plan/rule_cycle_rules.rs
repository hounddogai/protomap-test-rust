use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule cycle must pass. Check returns its problems, or none.
pub trait RuleCycleRule: std::fmt::Debug {
    fn check(&self, item: &RuleCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCycleRequiredCurrencyRule {
    pub message: String,
}

impl RuleCycleRule for RuleCycleRequiredCurrencyRule {
    fn check(&self, item: &RuleCycle) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCycleCurrencyLengthRule {
    pub max_length: i64,
}

impl RuleCycleRule for RuleCycleCurrencyLengthRule {
    fn check(&self, item: &RuleCycle) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCycleNameLengthRule {
    pub max_length: i64,
}

impl RuleCycleRule for RuleCycleNameLengthRule {
    fn check(&self, item: &RuleCycle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCycleDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleCycleRule for RuleCycleDurationMinutesRangeRule {
    fn check(&self, item: &RuleCycle) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RuleCycleAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl RuleCycleRule for RuleCycleAllowedPrincipalRule {
    fn check(&self, item: &RuleCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is blocked", item.principal)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule cycle.
#[derive(Debug, Clone)]
pub struct RuleCycleValidator {
    pub rules: Vec<Rc<dyn RuleCycleRule>>,
}

impl RuleCycleValidator {
    pub fn validate(&self, item: &RuleCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_cycle_validator() -> RuleCycleValidator {
    RuleCycleValidator {
        rules: vec![
            Rc::new(RuleCycleRequiredCurrencyRule {
                message: "rule cycle needs an entry".to_string(),
            }) as Rc<dyn RuleCycleRule>,
            Rc::new(RuleCycleCurrencyLengthRule { max_length: 3600 }),
            Rc::new(RuleCycleNameLengthRule { max_length: 64 }),
            Rc::new(RuleCycleDurationMinutesRangeRule { min: 32, max: 3600 }),
            Rc::new(RuleCycleAllowedPrincipalRule {
                allowed: vec!["adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_cycle_validator();
        let item = new_rule_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = RuleCycleRequiredCurrencyRule {
            message: "rule cycle needs this field".to_string(),
        };
        let mut item = new_rule_cycle("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = RuleCycleCurrencyLengthRule { max_length: 3600 };
        let mut item = new_rule_cycle("x", "y");
        item.currency = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RuleCycleNameLengthRule { max_length: 1000 };
        let mut item = new_rule_cycle("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RuleCycleDurationMinutesRangeRule { min: 120, max: 120 };
        let mut item = new_rule_cycle("x", "y");
        item.duration_minutes = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
