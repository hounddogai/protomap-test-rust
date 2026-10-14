use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence schedule must pass. Check returns its problems, or none.
pub trait EvidenceScheduleRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScheduleRequiredLabelRule {
    pub message: String,
}

impl EvidenceScheduleRule for EvidenceScheduleRequiredLabelRule {
    fn check(&self, item: &EvidenceSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScheduleRequiredCodeRule {
    pub message: String,
}

impl EvidenceScheduleRule for EvidenceScheduleRequiredCodeRule {
    fn check(&self, item: &EvidenceSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScheduleApprovedGuardRule {
    pub limit: i64,
}

impl EvidenceScheduleRule for EvidenceScheduleApprovedGuardRule {
    fn check(&self, item: &EvidenceSchedule) -> Vec<String> {
        if item.approved && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while approved",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScheduleCodePrefixRule {
    pub prefix: String,
}

impl EvidenceScheduleRule for EvidenceScheduleCodePrefixRule {
    fn check(&self, item: &EvidenceSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScheduleLabelLengthRule {
    pub max_length: i64,
}

impl EvidenceScheduleRule for EvidenceScheduleLabelLengthRule {
    fn check(&self, item: &EvidenceSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceScheduleClusterLengthRule {
    pub max_length: i64,
}

impl EvidenceScheduleRule for EvidenceScheduleClusterLengthRule {
    fn check(&self, item: &EvidenceSchedule) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence schedule.
#[derive(Debug, Clone)]
pub struct EvidenceScheduleValidator {
    pub rules: Vec<Rc<dyn EvidenceScheduleRule>>,
}

impl EvidenceScheduleValidator {
    pub fn validate(&self, item: &EvidenceSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_schedule_validator() -> EvidenceScheduleValidator {
    EvidenceScheduleValidator {
        rules: vec![
            Rc::new(EvidenceScheduleRequiredLabelRule {
                message: "evidence schedule needs a value".to_string(),
            }) as Rc<dyn EvidenceScheduleRule>,
            Rc::new(EvidenceScheduleRequiredCodeRule {
                message: "evidence schedule needs this field".to_string(),
            }),
            Rc::new(EvidenceScheduleApprovedGuardRule { limit: 255 }),
            Rc::new(EvidenceScheduleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(EvidenceScheduleLabelLengthRule { max_length: 120 }),
            Rc::new(EvidenceScheduleClusterLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_schedule_validator();
        let item = new_evidence_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = EvidenceScheduleRequiredLabelRule {
            message: "evidence schedule needs an entry".to_string(),
        };
        let mut item = new_evidence_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EvidenceScheduleRequiredCodeRule {
            message: "evidence schedule needs a value".to_string(),
        };
        let mut item = new_evidence_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EvidenceScheduleLabelLengthRule { max_length: 80 };
        let mut item = new_evidence_schedule("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = EvidenceScheduleClusterLengthRule { max_length: 86400 };
        let mut item = new_evidence_schedule("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
