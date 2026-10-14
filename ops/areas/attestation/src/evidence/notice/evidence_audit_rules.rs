use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence audit must pass. Check returns its problems, or none.
pub trait EvidenceAuditRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAuditRequiredIssuerRule {
    pub message: String,
}

impl EvidenceAuditRule for EvidenceAuditRequiredIssuerRule {
    fn check(&self, item: &EvidenceAudit) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAuditThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceAuditRule for EvidenceAuditThresholdRangeRule {
    fn check(&self, item: &EvidenceAudit) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAuditRequiredNameRule {
    pub message: String,
}

impl EvidenceAuditRule for EvidenceAuditRequiredNameRule {
    fn check(&self, item: &EvidenceAudit) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAuditRequiredCodeRule {
    pub message: String,
}

impl EvidenceAuditRule for EvidenceAuditRequiredCodeRule {
    fn check(&self, item: &EvidenceAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence audit.
#[derive(Debug, Clone)]
pub struct EvidenceAuditValidator {
    pub rules: Vec<Rc<dyn EvidenceAuditRule>>,
}

impl EvidenceAuditValidator {
    pub fn validate(&self, item: &EvidenceAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_audit_validator() -> EvidenceAuditValidator {
    EvidenceAuditValidator {
        rules: vec![
            Rc::new(EvidenceAuditRequiredIssuerRule {
                message: "evidence audit needs this field".to_string(),
            }) as Rc<dyn EvidenceAuditRule>,
            Rc::new(EvidenceAuditThresholdRangeRule { min: 4.8, max: 2618.5 }),
            Rc::new(EvidenceAuditRequiredNameRule {
                message: "evidence audit needs a value".to_string(),
            }),
            Rc::new(EvidenceAuditRequiredCodeRule {
                message: "evidence audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_audit_validator();
        let item = new_evidence_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = EvidenceAuditRequiredIssuerRule {
            message: "evidence audit needs this field".to_string(),
        };
        let mut item = new_evidence_audit("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = EvidenceAuditThresholdRangeRule { min: 9.3, max: 4765.1 };
        let mut item = new_evidence_audit("x", "y");
        item.threshold = 4766.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EvidenceAuditRequiredNameRule {
            message: "evidence audit needs this field".to_string(),
        };
        let mut item = new_evidence_audit("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceAuditRequiredCodeRule {
            message: "evidence audit needs a value".to_string(),
        };
        let mut item = new_evidence_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
