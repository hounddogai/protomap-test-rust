use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence record must pass. Check returns its problems, or none.
pub trait EvidenceRecordRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRecordRequiredLabelRule {
    pub message: String,
}

impl EvidenceRecordRule for EvidenceRecordRequiredLabelRule {
    fn check(&self, item: &EvidenceRecord) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRecordRequiredCodeRule {
    pub message: String,
}

impl EvidenceRecordRule for EvidenceRecordRequiredCodeRule {
    fn check(&self, item: &EvidenceRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRecordRequiredTenantIdRule {
    pub message: String,
}

impl EvidenceRecordRule for EvidenceRecordRequiredTenantIdRule {
    fn check(&self, item: &EvidenceRecord) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRecordRequiredCurrencyRule {
    pub message: String,
}

impl EvidenceRecordRule for EvidenceRecordRequiredCurrencyRule {
    fn check(&self, item: &EvidenceRecord) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence record.
#[derive(Debug, Clone)]
pub struct EvidenceRecordValidator {
    pub rules: Vec<Rc<dyn EvidenceRecordRule>>,
}

impl EvidenceRecordValidator {
    pub fn validate(&self, item: &EvidenceRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_record_validator() -> EvidenceRecordValidator {
    EvidenceRecordValidator {
        rules: vec![
            Rc::new(EvidenceRecordRequiredLabelRule {
                message: "evidence record needs an entry".to_string(),
            }) as Rc<dyn EvidenceRecordRule>,
            Rc::new(EvidenceRecordRequiredCodeRule {
                message: "evidence record needs an entry".to_string(),
            }),
            Rc::new(EvidenceRecordRequiredTenantIdRule {
                message: "evidence record needs this field".to_string(),
            }),
            Rc::new(EvidenceRecordRequiredCurrencyRule {
                message: "evidence record needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_record_validator();
        let item = new_evidence_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EvidenceRecordRequiredLabelRule {
            message: "evidence record needs a value".to_string(),
        };
        let mut item = new_evidence_record("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceRecordRequiredCodeRule {
            message: "evidence record needs a value".to_string(),
        };
        let mut item = new_evidence_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = EvidenceRecordRequiredTenantIdRule {
            message: "evidence record needs a value".to_string(),
        };
        let mut item = new_evidence_record("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = EvidenceRecordRequiredCurrencyRule {
            message: "evidence record needs a value".to_string(),
        };
        let mut item = new_evidence_record("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
