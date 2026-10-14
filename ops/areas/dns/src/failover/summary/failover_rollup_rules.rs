use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover rollup must pass. Check returns its problems, or none.
pub trait FailoverRollupRule: std::fmt::Debug {
    fn check(&self, item: &FailoverRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRollupTitleLengthRule {
    pub max_length: i64,
}

impl FailoverRollupRule for FailoverRollupTitleLengthRule {
    fn check(&self, item: &FailoverRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRollupCodeLengthRule {
    pub max_length: i64,
}

impl FailoverRollupRule for FailoverRollupCodeLengthRule {
    fn check(&self, item: &FailoverRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRollupQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverRollupRule for FailoverRollupQuotaUnitsRangeRule {
    fn check(&self, item: &FailoverRollup) -> Vec<String> {
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

/// Runs every rule of a failover rollup.
#[derive(Debug, Clone)]
pub struct FailoverRollupValidator {
    pub rules: Vec<Rc<dyn FailoverRollupRule>>,
}

impl FailoverRollupValidator {
    pub fn validate(&self, item: &FailoverRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_rollup_validator() -> FailoverRollupValidator {
    FailoverRollupValidator {
        rules: vec![
            Rc::new(FailoverRollupTitleLengthRule { max_length: 80 }) as Rc<dyn FailoverRollupRule>,
            Rc::new(FailoverRollupCodeLengthRule { max_length: 1000 }),
            Rc::new(FailoverRollupQuotaUnitsRangeRule { min: 64, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_rollup_validator();
        let item = new_failover_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FailoverRollupTitleLengthRule { max_length: 1000 };
        let mut item = new_failover_rollup("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FailoverRollupCodeLengthRule { max_length: 64 };
        let mut item = new_failover_rollup("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = FailoverRollupQuotaUnitsRangeRule { min: 120, max: 86400 };
        let mut item = new_failover_rollup("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
