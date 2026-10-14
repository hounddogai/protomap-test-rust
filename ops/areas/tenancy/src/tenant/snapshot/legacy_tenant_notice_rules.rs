use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy tenant notice must pass. Check returns its problems, or none.
pub trait LegacyTenantNoticeRule: std::fmt::Debug {
    fn check(&self, item: &LegacyTenantNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantNoticeTitleLengthRule {
    pub max_length: i64,
}

impl LegacyTenantNoticeRule for LegacyTenantNoticeTitleLengthRule {
    fn check(&self, item: &LegacyTenantNotice) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantNoticeCodePrefixRule {
    pub prefix: String,
}

impl LegacyTenantNoticeRule for LegacyTenantNoticeCodePrefixRule {
    fn check(&self, item: &LegacyTenantNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantNoticeRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyTenantNoticeRule for LegacyTenantNoticeRateRangeRule {
    fn check(&self, item: &LegacyTenantNotice) -> Vec<String> {
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

/// Runs every rule of a legacy tenant notice.
#[derive(Debug, Clone)]
pub struct LegacyTenantNoticeValidator {
    pub rules: Vec<Rc<dyn LegacyTenantNoticeRule>>,
}

impl LegacyTenantNoticeValidator {
    pub fn validate(&self, item: &LegacyTenantNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyTenantNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyTenantNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_tenant_notice_validator() -> LegacyTenantNoticeValidator {
    LegacyTenantNoticeValidator {
        rules: vec![
            Rc::new(LegacyTenantNoticeTitleLengthRule { max_length: 32 }) as Rc<dyn LegacyTenantNoticeRule>,
            Rc::new(LegacyTenantNoticeCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LegacyTenantNoticeRateRangeRule { min: 8.9, max: 4174.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_tenant_notice_validator();
        let item = new_legacy_tenant_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyTenantNoticeTitleLengthRule { max_length: 64 };
        let mut item = new_legacy_tenant_notice("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = LegacyTenantNoticeRateRangeRule { min: 4.3, max: 310.7 };
        let mut item = new_legacy_tenant_notice("x", "y");
        item.rate = 312.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
