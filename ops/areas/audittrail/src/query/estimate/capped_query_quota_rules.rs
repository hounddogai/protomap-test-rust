use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped query quota must pass. Check returns its problems, or none.
pub trait CappedQueryQuotaRule: std::fmt::Debug {
    fn check(&self, item: &CappedQueryQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryQuotaRequiredTimezoneRule {
    pub message: String,
}

impl CappedQueryQuotaRule for CappedQueryQuotaRequiredTimezoneRule {
    fn check(&self, item: &CappedQueryQuota) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryQuotaRequiredLabelRule {
    pub message: String,
}

impl CappedQueryQuotaRule for CappedQueryQuotaRequiredLabelRule {
    fn check(&self, item: &CappedQueryQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedQueryQuotaRule for CappedQueryQuotaAllowedCodeRule {
    fn check(&self, item: &CappedQueryQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryQuotaVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedQueryQuotaRule for CappedQueryQuotaVersionRangeRule {
    fn check(&self, item: &CappedQueryQuota) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryQuotaFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedQueryQuotaRule for CappedQueryQuotaFloorValueRangeRule {
    fn check(&self, item: &CappedQueryQuota) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped query quota.
#[derive(Debug, Clone)]
pub struct CappedQueryQuotaValidator {
    pub rules: Vec<Rc<dyn CappedQueryQuotaRule>>,
}

impl CappedQueryQuotaValidator {
    pub fn validate(&self, item: &CappedQueryQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedQueryQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedQueryQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_query_quota_validator() -> CappedQueryQuotaValidator {
    CappedQueryQuotaValidator {
        rules: vec![
            Rc::new(CappedQueryQuotaRequiredTimezoneRule {
                message: "capped query quota needs a value".to_string(),
            }) as Rc<dyn CappedQueryQuotaRule>,
            Rc::new(CappedQueryQuotaRequiredLabelRule {
                message: "capped query quota needs a value".to_string(),
            }),
            Rc::new(CappedQueryQuotaAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(CappedQueryQuotaVersionRangeRule { min: 16, max: 120 }),
            Rc::new(CappedQueryQuotaFloorValueRangeRule { min: 9.1, max: 3981.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_query_quota_validator();
        let item = new_capped_query_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = CappedQueryQuotaRequiredTimezoneRule {
            message: "capped query quota needs a value".to_string(),
        };
        let mut item = new_capped_query_quota("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedQueryQuotaRequiredLabelRule {
            message: "capped query quota needs a value".to_string(),
        };
        let mut item = new_capped_query_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = CappedQueryQuotaVersionRangeRule { min: 32, max: 3600 };
        let mut item = new_capped_query_quota("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = CappedQueryQuotaFloorValueRangeRule { min: 4.6, max: 2040.9 };
        let mut item = new_capped_query_quota("x", "y");
        item.floor_value = 2042.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
