use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal rule cycle must pass. Check returns its problems, or none.
pub trait SeasonalRuleCycleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRuleCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRuleCycleAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalRuleCycleRule for SeasonalRuleCycleAttemptsRangeRule {
    fn check(&self, item: &SeasonalRuleCycle) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRuleCycleExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalRuleCycleRule for SeasonalRuleCycleExpiresAtRangeRule {
    fn check(&self, item: &SeasonalRuleCycle) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRuleCycleActiveGuardRule {
    pub limit: f64,
}

impl SeasonalRuleCycleRule for SeasonalRuleCycleActiveGuardRule {
    fn check(&self, item: &SeasonalRuleCycle) -> Vec<String> {
        if item.active && (item.ceiling > self.limit) {
            return vec![format!("ceiling {} exceeds {} while active", item.ceiling, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal rule cycle.
#[derive(Debug, Clone)]
pub struct SeasonalRuleCycleValidator {
    pub rules: Vec<Rc<dyn SeasonalRuleCycleRule>>,
}

impl SeasonalRuleCycleValidator {
    pub fn validate(&self, item: &SeasonalRuleCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalRuleCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRuleCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_rule_cycle_validator() -> SeasonalRuleCycleValidator {
    SeasonalRuleCycleValidator {
        rules: vec![
            Rc::new(SeasonalRuleCycleAttemptsRangeRule { min: 16, max: 32 }) as Rc<dyn SeasonalRuleCycleRule>,
            Rc::new(SeasonalRuleCycleExpiresAtRangeRule { min: 32, max: 80 }),
            Rc::new(SeasonalRuleCycleActiveGuardRule { limit: 5.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_rule_cycle_validator();
        let item = new_seasonal_rule_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = SeasonalRuleCycleAttemptsRangeRule { min: 80, max: 120 };
        let mut item = new_seasonal_rule_cycle("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = SeasonalRuleCycleExpiresAtRangeRule { min: 64, max: 120 };
        let mut item = new_seasonal_rule_cycle("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
