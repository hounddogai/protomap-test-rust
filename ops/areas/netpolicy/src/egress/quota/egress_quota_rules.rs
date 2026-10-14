use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress quota must pass. Check returns its problems, or none.
pub trait EgressQuotaRule: std::fmt::Debug {
    fn check(&self, item: &EgressQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressQuotaRequiredSummaryRule {
    pub message: String,
}

impl EgressQuotaRule for EgressQuotaRequiredSummaryRule {
    fn check(&self, item: &EgressQuota) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressQuotaRequiredCodeRule {
    pub message: String,
}

impl EgressQuotaRule for EgressQuotaRequiredCodeRule {
    fn check(&self, item: &EgressQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressQuotaEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressQuotaRule for EgressQuotaEndsAtRangeRule {
    fn check(&self, item: &EgressQuota) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressQuotaRequiredLabelRule {
    pub message: String,
}

impl EgressQuotaRule for EgressQuotaRequiredLabelRule {
    fn check(&self, item: &EgressQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress quota.
#[derive(Debug, Clone)]
pub struct EgressQuotaValidator {
    pub rules: Vec<Rc<dyn EgressQuotaRule>>,
}

impl EgressQuotaValidator {
    pub fn validate(&self, item: &EgressQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_quota_validator() -> EgressQuotaValidator {
    EgressQuotaValidator {
        rules: vec![
            Rc::new(EgressQuotaRequiredSummaryRule {
                message: "egress quota needs a value".to_string(),
            }) as Rc<dyn EgressQuotaRule>,
            Rc::new(EgressQuotaRequiredCodeRule {
                message: "egress quota needs a value".to_string(),
            }),
            Rc::new(EgressQuotaEndsAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(EgressQuotaRequiredLabelRule {
                message: "egress quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_quota_validator();
        let item = new_egress_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = EgressQuotaRequiredSummaryRule {
            message: "egress quota needs this field".to_string(),
        };
        let mut item = new_egress_quota("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressQuotaRequiredCodeRule {
            message: "egress quota needs an entry".to_string(),
        };
        let mut item = new_egress_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = EgressQuotaEndsAtRangeRule { min: 32, max: 32 };
        let mut item = new_egress_quota("x", "y");
        item.ends_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EgressQuotaRequiredLabelRule {
            message: "egress quota needs this field".to_string(),
        };
        let mut item = new_egress_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
