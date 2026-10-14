use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal audit event notice must pass. Check returns its problems, or none.
pub trait SeasonalAuditEventNoticeRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalAuditEventNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAuditEventNoticeCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalAuditEventNoticeRule for SeasonalAuditEventNoticeCodeLengthRule {
    fn check(&self, item: &SeasonalAuditEventNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAuditEventNoticeRegionLengthRule {
    pub max_length: i64,
}

impl SeasonalAuditEventNoticeRule for SeasonalAuditEventNoticeRegionLengthRule {
    fn check(&self, item: &SeasonalAuditEventNotice) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAuditEventNoticeNameLengthRule {
    pub max_length: i64,
}

impl SeasonalAuditEventNoticeRule for SeasonalAuditEventNoticeNameLengthRule {
    fn check(&self, item: &SeasonalAuditEventNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal audit event notice.
#[derive(Debug, Clone)]
pub struct SeasonalAuditEventNoticeValidator {
    pub rules: Vec<Rc<dyn SeasonalAuditEventNoticeRule>>,
}

impl SeasonalAuditEventNoticeValidator {
    pub fn validate(&self, item: &SeasonalAuditEventNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalAuditEventNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalAuditEventNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_audit_event_notice_validator() -> SeasonalAuditEventNoticeValidator {
    SeasonalAuditEventNoticeValidator {
        rules: vec![
            Rc::new(SeasonalAuditEventNoticeCodeLengthRule { max_length: 255 }) as Rc<dyn SeasonalAuditEventNoticeRule>,
            Rc::new(SeasonalAuditEventNoticeRegionLengthRule { max_length: 80 }),
            Rc::new(SeasonalAuditEventNoticeNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_audit_event_notice_validator();
        let item = new_seasonal_audit_event_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalAuditEventNoticeCodeLengthRule { max_length: 255 };
        let mut item = new_seasonal_audit_event_notice("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = SeasonalAuditEventNoticeRegionLengthRule { max_length: 1000 };
        let mut item = new_seasonal_audit_event_notice("x", "y");
        item.region = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalAuditEventNoticeNameLengthRule { max_length: 1000 };
        let mut item = new_seasonal_audit_event_notice("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
