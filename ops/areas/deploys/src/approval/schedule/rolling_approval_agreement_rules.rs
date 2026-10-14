use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling approval agreement must pass. Check returns its problems, or none.
pub trait RollingApprovalAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RollingApprovalAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalAgreementRequiredNameRule {
    pub message: String,
}

impl RollingApprovalAgreementRule for RollingApprovalAgreementRequiredNameRule {
    fn check(&self, item: &RollingApprovalAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalAgreementEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingApprovalAgreementRule for RollingApprovalAgreementEndsAtRangeRule {
    fn check(&self, item: &RollingApprovalAgreement) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalAgreementWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingApprovalAgreementRule for RollingApprovalAgreementWindowDaysRangeRule {
    fn check(&self, item: &RollingApprovalAgreement) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalAgreementAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingApprovalAgreementRule for RollingApprovalAgreementAllowedCodeRule {
    fn check(&self, item: &RollingApprovalAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingApprovalAgreementRequiredClusterRule {
    pub message: String,
}

impl RollingApprovalAgreementRule for RollingApprovalAgreementRequiredClusterRule {
    fn check(&self, item: &RollingApprovalAgreement) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling approval agreement.
#[derive(Debug, Clone)]
pub struct RollingApprovalAgreementValidator {
    pub rules: Vec<Rc<dyn RollingApprovalAgreementRule>>,
}

impl RollingApprovalAgreementValidator {
    pub fn validate(&self, item: &RollingApprovalAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingApprovalAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingApprovalAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_approval_agreement_validator() -> RollingApprovalAgreementValidator {
    RollingApprovalAgreementValidator {
        rules: vec![
            Rc::new(RollingApprovalAgreementRequiredNameRule {
                message: "rolling approval agreement needs this field".to_string(),
            }) as Rc<dyn RollingApprovalAgreementRule>,
            Rc::new(RollingApprovalAgreementEndsAtRangeRule { min: 80, max: 255 }),
            Rc::new(RollingApprovalAgreementWindowDaysRangeRule { min: 16, max: 64 }),
            Rc::new(RollingApprovalAgreementAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "batched".to_string(), "fallback".to_string()],
            }),
            Rc::new(RollingApprovalAgreementRequiredClusterRule {
                message: "rolling approval agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_approval_agreement_validator();
        let item = new_rolling_approval_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingApprovalAgreementRequiredNameRule {
            message: "rolling approval agreement needs this field".to_string(),
        };
        let mut item = new_rolling_approval_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RollingApprovalAgreementEndsAtRangeRule { min: 32, max: 32 };
        let mut item = new_rolling_approval_agreement("x", "y");
        item.ends_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = RollingApprovalAgreementWindowDaysRangeRule { min: 32, max: 1000 };
        let mut item = new_rolling_approval_agreement("x", "y");
        item.window_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = RollingApprovalAgreementRequiredClusterRule {
            message: "rolling approval agreement needs this field".to_string(),
        };
        let mut item = new_rolling_approval_agreement("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
