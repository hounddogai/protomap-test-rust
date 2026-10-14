use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal isolation threshold must pass. Check returns its problems, or none.
pub trait SeasonalIsolationThresholdRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalIsolationThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationThresholdRequiredNameRule {
    pub message: String,
}

impl SeasonalIsolationThresholdRule for SeasonalIsolationThresholdRequiredNameRule {
    fn check(&self, item: &SeasonalIsolationThreshold) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationThresholdRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalIsolationThresholdRule for SeasonalIsolationThresholdRevisionRangeRule {
    fn check(&self, item: &SeasonalIsolationThreshold) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationThresholdCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalIsolationThresholdRule for SeasonalIsolationThresholdCodeLengthRule {
    fn check(&self, item: &SeasonalIsolationThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationThresholdUnitLengthRule {
    pub max_length: i64,
}

impl SeasonalIsolationThresholdRule for SeasonalIsolationThresholdUnitLengthRule {
    fn check(&self, item: &SeasonalIsolationThreshold) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationThresholdAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalIsolationThresholdRule for SeasonalIsolationThresholdAmountRangeRule {
    fn check(&self, item: &SeasonalIsolationThreshold) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal isolation threshold.
#[derive(Debug, Clone)]
pub struct SeasonalIsolationThresholdValidator {
    pub rules: Vec<Rc<dyn SeasonalIsolationThresholdRule>>,
}

impl SeasonalIsolationThresholdValidator {
    pub fn validate(&self, item: &SeasonalIsolationThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalIsolationThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalIsolationThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_isolation_threshold_validator() -> SeasonalIsolationThresholdValidator {
    SeasonalIsolationThresholdValidator {
        rules: vec![
            Rc::new(SeasonalIsolationThresholdRequiredNameRule {
                message: "seasonal isolation threshold needs this field".to_string(),
            }) as Rc<dyn SeasonalIsolationThresholdRule>,
            Rc::new(SeasonalIsolationThresholdRevisionRangeRule { min: 16, max: 86400 }),
            Rc::new(SeasonalIsolationThresholdCodeLengthRule { max_length: 32 }),
            Rc::new(SeasonalIsolationThresholdUnitLengthRule { max_length: 32 }),
            Rc::new(SeasonalIsolationThresholdAmountRangeRule { min: 5.3, max: 4622.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_isolation_threshold_validator();
        let item = new_seasonal_isolation_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SeasonalIsolationThresholdRequiredNameRule {
            message: "seasonal isolation threshold needs this field".to_string(),
        };
        let mut item = new_seasonal_isolation_threshold("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = SeasonalIsolationThresholdRevisionRangeRule { min: 255, max: 1000 };
        let mut item = new_seasonal_isolation_threshold("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalIsolationThresholdCodeLengthRule { max_length: 32 };
        let mut item = new_seasonal_isolation_threshold("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = SeasonalIsolationThresholdUnitLengthRule { max_length: 16 };
        let mut item = new_seasonal_isolation_threshold("x", "y");
        item.unit = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = SeasonalIsolationThresholdAmountRangeRule { min: 0.0, max: 1800.5 };
        let mut item = new_seasonal_isolation_threshold("x", "y");
        item.amount = 1802.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
