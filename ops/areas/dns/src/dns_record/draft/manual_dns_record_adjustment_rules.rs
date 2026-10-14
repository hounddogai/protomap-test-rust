use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual dns record adjustment must pass. Check returns its problems, or none.
pub trait ManualDnsRecordAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &ManualDnsRecordAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl ManualDnsRecordAdjustmentRule for ManualDnsRecordAdjustmentCodePrefixRule {
    fn check(&self, item: &ManualDnsRecordAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl ManualDnsRecordAdjustmentRule for ManualDnsRecordAdjustmentLabelLengthRule {
    fn check(&self, item: &ManualDnsRecordAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl ManualDnsRecordAdjustmentRule for ManualDnsRecordAdjustmentCodeLengthRule {
    fn check(&self, item: &ManualDnsRecordAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual dns record adjustment.
#[derive(Debug, Clone)]
pub struct ManualDnsRecordAdjustmentValidator {
    pub rules: Vec<Rc<dyn ManualDnsRecordAdjustmentRule>>,
}

impl ManualDnsRecordAdjustmentValidator {
    pub fn validate(&self, item: &ManualDnsRecordAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualDnsRecordAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDnsRecordAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_dns_record_adjustment_validator() -> ManualDnsRecordAdjustmentValidator {
    ManualDnsRecordAdjustmentValidator {
        rules: vec![
            Rc::new(ManualDnsRecordAdjustmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn ManualDnsRecordAdjustmentRule>,
            Rc::new(ManualDnsRecordAdjustmentLabelLengthRule { max_length: 86400 }),
            Rc::new(ManualDnsRecordAdjustmentCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_dns_record_adjustment_validator();
        let item = new_manual_dns_record_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualDnsRecordAdjustmentLabelLengthRule { max_length: 255 };
        let mut item = new_manual_dns_record_adjustment("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualDnsRecordAdjustmentCodeLengthRule { max_length: 16 };
        let mut item = new_manual_dns_record_adjustment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
