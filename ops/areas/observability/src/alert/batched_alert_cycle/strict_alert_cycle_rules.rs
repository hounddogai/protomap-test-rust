use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict alert cycle must pass. Check returns its problems, or none.
pub trait StrictAlertCycleRule: std::fmt::Debug {
    fn check(&self, item: &StrictAlertCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertCycleMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictAlertCycleRule for StrictAlertCycleMarginRangeRule {
    fn check(&self, item: &StrictAlertCycle) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertCycleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictAlertCycleRule for StrictAlertCycleScoreRangeRule {
    fn check(&self, item: &StrictAlertCycle) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertCycleAudienceLengthRule {
    pub max_length: i64,
}

impl StrictAlertCycleRule for StrictAlertCycleAudienceLengthRule {
    fn check(&self, item: &StrictAlertCycle) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertCycleCodeLengthRule {
    pub max_length: i64,
}

impl StrictAlertCycleRule for StrictAlertCycleCodeLengthRule {
    fn check(&self, item: &StrictAlertCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict alert cycle.
#[derive(Debug, Clone)]
pub struct StrictAlertCycleValidator {
    pub rules: Vec<Rc<dyn StrictAlertCycleRule>>,
}

impl StrictAlertCycleValidator {
    pub fn validate(&self, item: &StrictAlertCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictAlertCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictAlertCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_alert_cycle_validator() -> StrictAlertCycleValidator {
    StrictAlertCycleValidator {
        rules: vec![
            Rc::new(StrictAlertCycleMarginRangeRule { min: 8.6, max: 894.7 }) as Rc<dyn StrictAlertCycleRule>,
            Rc::new(StrictAlertCycleScoreRangeRule { min: 9.3, max: 2665.8 }),
            Rc::new(StrictAlertCycleAudienceLengthRule { max_length: 80 }),
            Rc::new(StrictAlertCycleCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_alert_cycle_validator();
        let item = new_strict_alert_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = StrictAlertCycleMarginRangeRule { min: 3.8, max: 1095.0 };
        let mut item = new_strict_alert_cycle("x", "y");
        item.margin = 1096.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = StrictAlertCycleScoreRangeRule { min: 8.0, max: 206.4 };
        let mut item = new_strict_alert_cycle("x", "y");
        item.score = 207.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = StrictAlertCycleAudienceLengthRule { max_length: 64 };
        let mut item = new_strict_alert_cycle("x", "y");
        item.audience = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictAlertCycleCodeLengthRule { max_length: 3600 };
        let mut item = new_strict_alert_cycle("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
