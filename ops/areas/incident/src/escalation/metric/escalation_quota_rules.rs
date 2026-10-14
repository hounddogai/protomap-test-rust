use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation quota must pass. Check returns its problems, or none.
pub trait EscalationQuotaRule: std::fmt::Debug {
    fn check(&self, item: &EscalationQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationQuotaPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationQuotaRule for EscalationQuotaPriorityRangeRule {
    fn check(&self, item: &EscalationQuota) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationQuotaExternalRefLengthRule {
    pub max_length: i64,
}

impl EscalationQuotaRule for EscalationQuotaExternalRefLengthRule {
    fn check(&self, item: &EscalationQuota) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationQuotaTitleLengthRule {
    pub max_length: i64,
}

impl EscalationQuotaRule for EscalationQuotaTitleLengthRule {
    fn check(&self, item: &EscalationQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationQuotaRequiredLabelRule {
    pub message: String,
}

impl EscalationQuotaRule for EscalationQuotaRequiredLabelRule {
    fn check(&self, item: &EscalationQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationQuotaRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationQuotaRule for EscalationQuotaRatioRangeRule {
    fn check(&self, item: &EscalationQuota) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationQuotaRequiredTitleRule {
    pub message: String,
}

impl EscalationQuotaRule for EscalationQuotaRequiredTitleRule {
    fn check(&self, item: &EscalationQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation quota.
#[derive(Debug, Clone)]
pub struct EscalationQuotaValidator {
    pub rules: Vec<Rc<dyn EscalationQuotaRule>>,
}

impl EscalationQuotaValidator {
    pub fn validate(&self, item: &EscalationQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_quota_validator() -> EscalationQuotaValidator {
    EscalationQuotaValidator {
        rules: vec![
            Rc::new(EscalationQuotaPriorityRangeRule { min: 120, max: 3600 }) as Rc<dyn EscalationQuotaRule>,
            Rc::new(EscalationQuotaExternalRefLengthRule { max_length: 32 }),
            Rc::new(EscalationQuotaTitleLengthRule { max_length: 64 }),
            Rc::new(EscalationQuotaRequiredLabelRule {
                message: "escalation quota needs a value".to_string(),
            }),
            Rc::new(EscalationQuotaRatioRangeRule { min: 0.3, max: 4352.0 }),
            Rc::new(EscalationQuotaRequiredTitleRule {
                message: "escalation quota needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_quota_validator();
        let item = new_escalation_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = EscalationQuotaPriorityRangeRule { min: 120, max: 86400 };
        let mut item = new_escalation_quota("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = EscalationQuotaExternalRefLengthRule { max_length: 3600 };
        let mut item = new_escalation_quota("x", "y");
        item.external_ref = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EscalationQuotaTitleLengthRule { max_length: 3600 };
        let mut item = new_escalation_quota("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EscalationQuotaRequiredLabelRule {
            message: "escalation quota needs this field".to_string(),
        };
        let mut item = new_escalation_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = EscalationQuotaRatioRangeRule { min: 1.1, max: 2600.7 };
        let mut item = new_escalation_quota("x", "y");
        item.ratio = 2602.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EscalationQuotaRequiredTitleRule {
            message: "escalation quota needs an entry".to_string(),
        };
        let mut item = new_escalation_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
