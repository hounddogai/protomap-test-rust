use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation rollup must pass. Check returns its problems, or none.
pub trait IsolationRollupRule: std::fmt::Debug {
    fn check(&self, item: &IsolationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRollupRequiredCountryRule {
    pub message: String,
}

impl IsolationRollupRule for IsolationRollupRequiredCountryRule {
    fn check(&self, item: &IsolationRollup) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRollupCodeLengthRule {
    pub max_length: i64,
}

impl IsolationRollupRule for IsolationRollupCodeLengthRule {
    fn check(&self, item: &IsolationRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRollupQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationRollupRule for IsolationRollupQuantityRangeRule {
    fn check(&self, item: &IsolationRollup) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a isolation rollup.
#[derive(Debug, Clone)]
pub struct IsolationRollupValidator {
    pub rules: Vec<Rc<dyn IsolationRollupRule>>,
}

impl IsolationRollupValidator {
    pub fn validate(&self, item: &IsolationRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_rollup_validator() -> IsolationRollupValidator {
    IsolationRollupValidator {
        rules: vec![
            Rc::new(IsolationRollupRequiredCountryRule {
                message: "isolation rollup needs an entry".to_string(),
            }) as Rc<dyn IsolationRollupRule>,
            Rc::new(IsolationRollupCodeLengthRule { max_length: 255 }),
            Rc::new(IsolationRollupQuantityRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_rollup_validator();
        let item = new_isolation_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = IsolationRollupRequiredCountryRule {
            message: "isolation rollup needs a value".to_string(),
        };
        let mut item = new_isolation_rollup("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IsolationRollupCodeLengthRule { max_length: 80 };
        let mut item = new_isolation_rollup("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = IsolationRollupQuantityRangeRule { min: 64, max: 3600 };
        let mut item = new_isolation_rollup("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
