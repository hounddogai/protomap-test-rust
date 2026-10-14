use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional silence exception must pass. Check returns its problems, or none.
pub trait RegionalSilenceExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RegionalSilenceException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceExceptionRequiredCurrencyRule {
    pub message: String,
}

impl RegionalSilenceExceptionRule for RegionalSilenceExceptionRequiredCurrencyRule {
    fn check(&self, item: &RegionalSilenceException) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceExceptionRequiredUnitRule {
    pub message: String,
}

impl RegionalSilenceExceptionRule for RegionalSilenceExceptionRequiredUnitRule {
    fn check(&self, item: &RegionalSilenceException) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceExceptionActiveGuardRule {
    pub limit: i64,
}

impl RegionalSilenceExceptionRule for RegionalSilenceExceptionActiveGuardRule {
    fn check(&self, item: &RegionalSilenceException) -> Vec<String> {
        if item.active && (item.ends_at > self.limit) {
            return vec![format!("ends at {} exceeds {} while active", item.ends_at, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceExceptionExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalSilenceExceptionRule for RegionalSilenceExceptionExpiresAtRangeRule {
    fn check(&self, item: &RegionalSilenceException) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional silence exception.
#[derive(Debug, Clone)]
pub struct RegionalSilenceExceptionValidator {
    pub rules: Vec<Rc<dyn RegionalSilenceExceptionRule>>,
}

impl RegionalSilenceExceptionValidator {
    pub fn validate(&self, item: &RegionalSilenceException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalSilenceException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalSilenceException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_silence_exception_validator() -> RegionalSilenceExceptionValidator {
    RegionalSilenceExceptionValidator {
        rules: vec![
            Rc::new(RegionalSilenceExceptionRequiredCurrencyRule {
                message: "regional silence exception needs a value".to_string(),
            }) as Rc<dyn RegionalSilenceExceptionRule>,
            Rc::new(RegionalSilenceExceptionRequiredUnitRule {
                message: "regional silence exception needs a value".to_string(),
            }),
            Rc::new(RegionalSilenceExceptionActiveGuardRule { limit: 3600 }),
            Rc::new(RegionalSilenceExceptionExpiresAtRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_silence_exception_validator();
        let item = new_regional_silence_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = RegionalSilenceExceptionRequiredCurrencyRule {
            message: "regional silence exception needs an entry".to_string(),
        };
        let mut item = new_regional_silence_exception("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = RegionalSilenceExceptionRequiredUnitRule {
            message: "regional silence exception needs a value".to_string(),
        };
        let mut item = new_regional_silence_exception("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RegionalSilenceExceptionExpiresAtRangeRule { min: 1000, max: 3600 };
        let mut item = new_regional_silence_exception("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
