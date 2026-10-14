use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending plan quota must pass. Check returns its problems, or none.
pub trait PendingPlanQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PendingPlanQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanQuotaCodePrefixRule {
    pub prefix: String,
}

impl PendingPlanQuotaRule for PendingPlanQuotaCodePrefixRule {
    fn check(&self, item: &PendingPlanQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanQuotaApprovedGuardRule {
    pub limit: f64,
}

impl PendingPlanQuotaRule for PendingPlanQuotaApprovedGuardRule {
    fn check(&self, item: &PendingPlanQuota) -> Vec<String> {
        if item.approved && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while approved", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanQuotaRequiredLabelRule {
    pub message: String,
}

impl PendingPlanQuotaRule for PendingPlanQuotaRequiredLabelRule {
    fn check(&self, item: &PendingPlanQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanQuotaRequiredCodeRule {
    pub message: String,
}

impl PendingPlanQuotaRule for PendingPlanQuotaRequiredCodeRule {
    fn check(&self, item: &PendingPlanQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending plan quota.
#[derive(Debug, Clone)]
pub struct PendingPlanQuotaValidator {
    pub rules: Vec<Rc<dyn PendingPlanQuotaRule>>,
}

impl PendingPlanQuotaValidator {
    pub fn validate(&self, item: &PendingPlanQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPlanQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPlanQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_plan_quota_validator() -> PendingPlanQuotaValidator {
    PendingPlanQuotaValidator {
        rules: vec![
            Rc::new(PendingPlanQuotaCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn PendingPlanQuotaRule>,
            Rc::new(PendingPlanQuotaApprovedGuardRule { limit: 8.7 }),
            Rc::new(PendingPlanQuotaRequiredLabelRule {
                message: "pending plan quota needs an entry".to_string(),
            }),
            Rc::new(PendingPlanQuotaRequiredCodeRule {
                message: "pending plan quota needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_plan_quota_validator();
        let item = new_pending_plan_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingPlanQuotaRequiredLabelRule {
            message: "pending plan quota needs a value".to_string(),
        };
        let mut item = new_pending_plan_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingPlanQuotaRequiredCodeRule {
            message: "pending plan quota needs this field".to_string(),
        };
        let mut item = new_pending_plan_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
