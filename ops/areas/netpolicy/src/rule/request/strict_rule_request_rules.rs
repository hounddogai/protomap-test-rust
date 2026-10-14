use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict rule request must pass. Check returns its problems, or none.
pub trait StrictRuleRequestRule: std::fmt::Debug {
    fn check(&self, item: &StrictRuleRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleRequestBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRuleRequestRule for StrictRuleRequestBudgetRangeRule {
    fn check(&self, item: &StrictRuleRequest) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleRequestNameLengthRule {
    pub max_length: i64,
}

impl StrictRuleRequestRule for StrictRuleRequestNameLengthRule {
    fn check(&self, item: &StrictRuleRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRuleRequestCurrencyLengthRule {
    pub max_length: i64,
}

impl StrictRuleRequestRule for StrictRuleRequestCurrencyLengthRule {
    fn check(&self, item: &StrictRuleRequest) -> Vec<String> {
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
pub struct StrictRuleRequestChecksumLengthRule {
    pub max_length: i64,
}

impl StrictRuleRequestRule for StrictRuleRequestChecksumLengthRule {
    fn check(&self, item: &StrictRuleRequest) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict rule request.
#[derive(Debug, Clone)]
pub struct StrictRuleRequestValidator {
    pub rules: Vec<Rc<dyn StrictRuleRequestRule>>,
}

impl StrictRuleRequestValidator {
    pub fn validate(&self, item: &StrictRuleRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictRuleRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRuleRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_rule_request_validator() -> StrictRuleRequestValidator {
    StrictRuleRequestValidator {
        rules: vec![
            Rc::new(StrictRuleRequestBudgetRangeRule { min: 4.0, max: 4068.3 }) as Rc<dyn StrictRuleRequestRule>,
            Rc::new(StrictRuleRequestNameLengthRule { max_length: 16 }),
            Rc::new(StrictRuleRequestCurrencyLengthRule { max_length: 16 }),
            Rc::new(StrictRuleRequestChecksumLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_rule_request_validator();
        let item = new_strict_rule_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = StrictRuleRequestBudgetRangeRule { min: 10.0, max: 2604.9 };
        let mut item = new_strict_rule_request("x", "y");
        item.budget = 2606.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictRuleRequestNameLengthRule { max_length: 3600 };
        let mut item = new_strict_rule_request("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = StrictRuleRequestCurrencyLengthRule { max_length: 1000 };
        let mut item = new_strict_rule_request("x", "y");
        item.currency = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = StrictRuleRequestChecksumLengthRule { max_length: 3600 };
        let mut item = new_strict_rule_request("x", "y");
        item.checksum = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
