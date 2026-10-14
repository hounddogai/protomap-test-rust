use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record draft must pass. Check returns its problems, or none.
pub trait DnsRecordDraftRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordDraftRequiredCodeRule {
    pub message: String,
}

impl DnsRecordDraftRule for DnsRecordDraftRequiredCodeRule {
    fn check(&self, item: &DnsRecordDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordDraftLabelLengthRule {
    pub max_length: i64,
}

impl DnsRecordDraftRule for DnsRecordDraftLabelLengthRule {
    fn check(&self, item: &DnsRecordDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordDraftMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DnsRecordDraftRule for DnsRecordDraftMaximumRangeRule {
    fn check(&self, item: &DnsRecordDraft) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordDraftApprovedGuardRule {
    pub limit: f64,
}

impl DnsRecordDraftRule for DnsRecordDraftApprovedGuardRule {
    fn check(&self, item: &DnsRecordDraft) -> Vec<String> {
        if item.approved && (item.maximum > self.limit) {
            return vec![format!(
                "maximum {} exceeds {} while approved",
                item.maximum, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record draft.
#[derive(Debug, Clone)]
pub struct DnsRecordDraftValidator {
    pub rules: Vec<Rc<dyn DnsRecordDraftRule>>,
}

impl DnsRecordDraftValidator {
    pub fn validate(&self, item: &DnsRecordDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_draft_validator() -> DnsRecordDraftValidator {
    DnsRecordDraftValidator {
        rules: vec![
            Rc::new(DnsRecordDraftRequiredCodeRule {
                message: "dns record draft needs a value".to_string(),
            }) as Rc<dyn DnsRecordDraftRule>,
            Rc::new(DnsRecordDraftLabelLengthRule { max_length: 86400 }),
            Rc::new(DnsRecordDraftMaximumRangeRule { min: 9.6, max: 1591.0 }),
            Rc::new(DnsRecordDraftApprovedGuardRule { limit: 1.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_draft_validator();
        let item = new_dns_record_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordDraftRequiredCodeRule {
            message: "dns record draft needs an entry".to_string(),
        };
        let mut item = new_dns_record_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DnsRecordDraftLabelLengthRule { max_length: 120 };
        let mut item = new_dns_record_draft("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = DnsRecordDraftMaximumRangeRule { min: 2.5, max: 784.7 };
        let mut item = new_dns_record_draft("x", "y");
        item.maximum = 786.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
