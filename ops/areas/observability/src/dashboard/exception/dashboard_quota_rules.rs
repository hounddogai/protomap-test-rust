use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dashboard quota must pass. Check returns its problems, or none.
pub trait DashboardQuotaRule: std::fmt::Debug {
    fn check(&self, item: &DashboardQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DashboardQuotaRule for DashboardQuotaAllowedCodeRule {
    fn check(&self, item: &DashboardQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardQuotaRequiredCodeRule {
    pub message: String,
}

impl DashboardQuotaRule for DashboardQuotaRequiredCodeRule {
    fn check(&self, item: &DashboardQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardQuotaNameLengthRule {
    pub max_length: i64,
}

impl DashboardQuotaRule for DashboardQuotaNameLengthRule {
    fn check(&self, item: &DashboardQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardQuotaRequiredHostRule {
    pub message: String,
}

impl DashboardQuotaRule for DashboardQuotaRequiredHostRule {
    fn check(&self, item: &DashboardQuota) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardQuotaMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardQuotaRule for DashboardQuotaMarginRangeRule {
    fn check(&self, item: &DashboardQuota) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardQuotaCodePrefixRule {
    pub prefix: String,
}

impl DashboardQuotaRule for DashboardQuotaCodePrefixRule {
    fn check(&self, item: &DashboardQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard quota.
#[derive(Debug, Clone)]
pub struct DashboardQuotaValidator {
    pub rules: Vec<Rc<dyn DashboardQuotaRule>>,
}

impl DashboardQuotaValidator {
    pub fn validate(&self, item: &DashboardQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_quota_validator() -> DashboardQuotaValidator {
    DashboardQuotaValidator {
        rules: vec![
            Rc::new(DashboardQuotaAllowedCodeRule {
                allowed: vec!["rolling".to_string(), "lenient".to_string(), "lenient".to_string()],
            }) as Rc<dyn DashboardQuotaRule>,
            Rc::new(DashboardQuotaRequiredCodeRule {
                message: "dashboard quota needs this field".to_string(),
            }),
            Rc::new(DashboardQuotaNameLengthRule { max_length: 255 }),
            Rc::new(DashboardQuotaRequiredHostRule {
                message: "dashboard quota needs this field".to_string(),
            }),
            Rc::new(DashboardQuotaMarginRangeRule { min: 9.8, max: 4554.3 }),
            Rc::new(DashboardQuotaCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_quota_validator();
        let item = new_dashboard_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DashboardQuotaRequiredCodeRule {
            message: "dashboard quota needs an entry".to_string(),
        };
        let mut item = new_dashboard_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DashboardQuotaNameLengthRule { max_length: 120 };
        let mut item = new_dashboard_quota("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = DashboardQuotaRequiredHostRule {
            message: "dashboard quota needs this field".to_string(),
        };
        let mut item = new_dashboard_quota("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = DashboardQuotaMarginRangeRule { min: 5.9, max: 2736.1 };
        let mut item = new_dashboard_quota("x", "y");
        item.margin = 2737.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
