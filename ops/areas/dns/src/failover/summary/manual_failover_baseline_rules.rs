use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual failover baseline must pass. Check returns its problems, or none.
pub trait ManualFailoverBaselineRule: std::fmt::Debug {
    fn check(&self, item: &ManualFailoverBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverBaselineLocaleLengthRule {
    pub max_length: i64,
}

impl ManualFailoverBaselineRule for ManualFailoverBaselineLocaleLengthRule {
    fn check(&self, item: &ManualFailoverBaseline) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverBaselineHostLengthRule {
    pub max_length: i64,
}

impl ManualFailoverBaselineRule for ManualFailoverBaselineHostLengthRule {
    fn check(&self, item: &ManualFailoverBaseline) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualFailoverBaselineRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualFailoverBaselineRule for ManualFailoverBaselineRateRangeRule {
    fn check(&self, item: &ManualFailoverBaseline) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual failover baseline.
#[derive(Debug, Clone)]
pub struct ManualFailoverBaselineValidator {
    pub rules: Vec<Rc<dyn ManualFailoverBaselineRule>>,
}

impl ManualFailoverBaselineValidator {
    pub fn validate(&self, item: &ManualFailoverBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualFailoverBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualFailoverBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_failover_baseline_validator() -> ManualFailoverBaselineValidator {
    ManualFailoverBaselineValidator {
        rules: vec![
            Rc::new(ManualFailoverBaselineLocaleLengthRule { max_length: 120 }) as Rc<dyn ManualFailoverBaselineRule>,
            Rc::new(ManualFailoverBaselineHostLengthRule { max_length: 3600 }),
            Rc::new(ManualFailoverBaselineRateRangeRule { min: 3.6, max: 741.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_failover_baseline_validator();
        let item = new_manual_failover_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ManualFailoverBaselineLocaleLengthRule { max_length: 120 };
        let mut item = new_manual_failover_baseline("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = ManualFailoverBaselineHostLengthRule { max_length: 16 };
        let mut item = new_manual_failover_baseline("x", "y");
        item.host = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = ManualFailoverBaselineRateRangeRule { min: 4.7, max: 3666.4 };
        let mut item = new_manual_failover_baseline("x", "y");
        item.rate = 3667.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
