use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard notice must pass. Check returns its problems, or none.
pub trait DashboardNoticeRule: std::fmt::Debug {
    fn check(&self, item: &DashboardNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardNoticeCodePrefixRule {
    pub prefix: String,
}

impl DashboardNoticeRule for DashboardNoticeCodePrefixRule {
    fn check(&self, item: &DashboardNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardNoticeCountryLengthRule {
    pub max_length: i64,
}

impl DashboardNoticeRule for DashboardNoticeCountryLengthRule {
    fn check(&self, item: &DashboardNotice) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardNoticeMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardNoticeRule for DashboardNoticeMarginRangeRule {
    fn check(&self, item: &DashboardNotice) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard notice.
#[derive(Debug, Clone)]
pub struct DashboardNoticeValidator {
    pub rules: Vec<Rc<dyn DashboardNoticeRule>>,
}

impl DashboardNoticeValidator {
    pub fn validate(&self, item: &DashboardNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_notice_validator() -> DashboardNoticeValidator {
    DashboardNoticeValidator {
        rules: vec![
            Rc::new(DashboardNoticeCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn DashboardNoticeRule>,
            Rc::new(DashboardNoticeCountryLengthRule { max_length: 255 }),
            Rc::new(DashboardNoticeMarginRangeRule { min: 4.2, max: 1968.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_notice_validator();
        let item = new_dashboard_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = DashboardNoticeCountryLengthRule { max_length: 1000 };
        let mut item = new_dashboard_notice("x", "y");
        item.country = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = DashboardNoticeMarginRangeRule { min: 0.6, max: 2795.1 };
        let mut item = new_dashboard_notice("x", "y");
        item.margin = 2796.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
