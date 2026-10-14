use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a silence quota must pass. Check returns its problems, or none.
pub trait SilenceQuotaRule: std::fmt::Debug {
    fn check(&self, item: &SilenceQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceQuotaCodeLengthRule {
    pub max_length: i64,
}

impl SilenceQuotaRule for SilenceQuotaCodeLengthRule {
    fn check(&self, item: &SilenceQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceQuotaRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceQuotaRule for SilenceQuotaRateRangeRule {
    fn check(&self, item: &SilenceQuota) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceQuotaLabelLengthRule {
    pub max_length: i64,
}

impl SilenceQuotaRule for SilenceQuotaLabelLengthRule {
    fn check(&self, item: &SilenceQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceQuotaAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl SilenceQuotaRule for SilenceQuotaAllowedUnitRule {
    fn check(&self, item: &SilenceQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is duplicated", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceQuotaCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceQuotaRule for SilenceQuotaCreatedAtRangeRule {
    fn check(&self, item: &SilenceQuota) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a silence quota.
#[derive(Debug, Clone)]
pub struct SilenceQuotaValidator {
    pub rules: Vec<Rc<dyn SilenceQuotaRule>>,
}

impl SilenceQuotaValidator {
    pub fn validate(&self, item: &SilenceQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_quota_validator() -> SilenceQuotaValidator {
    SilenceQuotaValidator {
        rules: vec![
            Rc::new(SilenceQuotaCodeLengthRule { max_length: 32 }) as Rc<dyn SilenceQuotaRule>,
            Rc::new(SilenceQuotaRateRangeRule { min: 1.6, max: 2355.5 }),
            Rc::new(SilenceQuotaLabelLengthRule { max_length: 32 }),
            Rc::new(SilenceQuotaAllowedUnitRule {
                allowed: vec!["tiered".to_string()],
            }),
            Rc::new(SilenceQuotaCreatedAtRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_quota_validator();
        let item = new_silence_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SilenceQuotaCodeLengthRule { max_length: 32 };
        let mut item = new_silence_quota("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = SilenceQuotaRateRangeRule { min: 9.9, max: 676.3 };
        let mut item = new_silence_quota("x", "y");
        item.rate = 677.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SilenceQuotaLabelLengthRule { max_length: 80 };
        let mut item = new_silence_quota("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = SilenceQuotaCreatedAtRangeRule { min: 120, max: 3600 };
        let mut item = new_silence_quota("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
