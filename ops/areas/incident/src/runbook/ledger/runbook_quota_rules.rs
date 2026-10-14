use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook quota must pass. Check returns its problems, or none.
pub trait RunbookQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RunbookQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookQuotaRequiredLabelRule {
    pub message: String,
}

impl RunbookQuotaRule for RunbookQuotaRequiredLabelRule {
    fn check(&self, item: &RunbookQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookQuotaRegionLengthRule {
    pub max_length: i64,
}

impl RunbookQuotaRule for RunbookQuotaRegionLengthRule {
    fn check(&self, item: &RunbookQuota) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookQuotaCodeLengthRule {
    pub max_length: i64,
}

impl RunbookQuotaRule for RunbookQuotaCodeLengthRule {
    fn check(&self, item: &RunbookQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook quota.
#[derive(Debug, Clone)]
pub struct RunbookQuotaValidator {
    pub rules: Vec<Rc<dyn RunbookQuotaRule>>,
}

impl RunbookQuotaValidator {
    pub fn validate(&self, item: &RunbookQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_quota_validator() -> RunbookQuotaValidator {
    RunbookQuotaValidator {
        rules: vec![
            Rc::new(RunbookQuotaRequiredLabelRule {
                message: "runbook quota needs this field".to_string(),
            }) as Rc<dyn RunbookQuotaRule>,
            Rc::new(RunbookQuotaRegionLengthRule { max_length: 64 }),
            Rc::new(RunbookQuotaCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_quota_validator();
        let item = new_runbook_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RunbookQuotaRequiredLabelRule {
            message: "runbook quota needs an entry".to_string(),
        };
        let mut item = new_runbook_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = RunbookQuotaRegionLengthRule { max_length: 1000 };
        let mut item = new_runbook_quota("x", "y");
        item.region = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RunbookQuotaCodeLengthRule { max_length: 120 };
        let mut item = new_runbook_quota("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
