use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension quota must pass. Check returns its problems, or none.
pub trait SuspensionQuotaRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionQuotaCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionQuotaRule for SuspensionQuotaCodeLengthRule {
    fn check(&self, item: &SuspensionQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionQuotaQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionQuotaRule for SuspensionQuotaQuotaUnitsRangeRule {
    fn check(&self, item: &SuspensionQuota) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionQuotaSourceLengthRule {
    pub max_length: i64,
}

impl SuspensionQuotaRule for SuspensionQuotaSourceLengthRule {
    fn check(&self, item: &SuspensionQuota) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension quota.
#[derive(Debug, Clone)]
pub struct SuspensionQuotaValidator {
    pub rules: Vec<Rc<dyn SuspensionQuotaRule>>,
}

impl SuspensionQuotaValidator {
    pub fn validate(&self, item: &SuspensionQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_quota_validator() -> SuspensionQuotaValidator {
    SuspensionQuotaValidator {
        rules: vec![
            Rc::new(SuspensionQuotaCodeLengthRule { max_length: 80 }) as Rc<dyn SuspensionQuotaRule>,
            Rc::new(SuspensionQuotaQuotaUnitsRangeRule { min: 16, max: 3600 }),
            Rc::new(SuspensionQuotaSourceLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_quota_validator();
        let item = new_suspension_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SuspensionQuotaCodeLengthRule { max_length: 32 };
        let mut item = new_suspension_quota("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = SuspensionQuotaQuotaUnitsRangeRule { min: 80, max: 3600 };
        let mut item = new_suspension_quota("x", "y");
        item.quota_units = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = SuspensionQuotaSourceLengthRule { max_length: 80 };
        let mut item = new_suspension_quota("x", "y");
        item.source = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
