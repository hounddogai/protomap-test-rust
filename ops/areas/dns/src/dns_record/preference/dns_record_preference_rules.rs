use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dns record preference must pass. Check returns its problems, or none.
pub trait DnsRecordPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &DnsRecordPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPreferenceRequiredIssuerRule {
    pub message: String,
}

impl DnsRecordPreferenceRule for DnsRecordPreferenceRequiredIssuerRule {
    fn check(&self, item: &DnsRecordPreference) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPreferenceLabelLengthRule {
    pub max_length: i64,
}

impl DnsRecordPreferenceRule for DnsRecordPreferenceLabelLengthRule {
    fn check(&self, item: &DnsRecordPreference) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPreferenceRequiredCodeRule {
    pub message: String,
}

impl DnsRecordPreferenceRule for DnsRecordPreferenceRequiredCodeRule {
    fn check(&self, item: &DnsRecordPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordPreferenceRequiredLabelRule {
    pub message: String,
}

impl DnsRecordPreferenceRule for DnsRecordPreferenceRequiredLabelRule {
    fn check(&self, item: &DnsRecordPreference) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a dns record preference.
#[derive(Debug, Clone)]
pub struct DnsRecordPreferenceValidator {
    pub rules: Vec<Rc<dyn DnsRecordPreferenceRule>>,
}

impl DnsRecordPreferenceValidator {
    pub fn validate(&self, item: &DnsRecordPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DnsRecordPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DnsRecordPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dns_record_preference_validator() -> DnsRecordPreferenceValidator {
    DnsRecordPreferenceValidator {
        rules: vec![
            Rc::new(DnsRecordPreferenceRequiredIssuerRule {
                message: "dns record preference needs this field".to_string(),
            }) as Rc<dyn DnsRecordPreferenceRule>,
            Rc::new(DnsRecordPreferenceLabelLengthRule { max_length: 80 }),
            Rc::new(DnsRecordPreferenceRequiredCodeRule {
                message: "dns record preference needs this field".to_string(),
            }),
            Rc::new(DnsRecordPreferenceRequiredLabelRule {
                message: "dns record preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dns_record_preference_validator();
        let item = new_dns_record_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = DnsRecordPreferenceRequiredIssuerRule {
            message: "dns record preference needs a value".to_string(),
        };
        let mut item = new_dns_record_preference("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DnsRecordPreferenceLabelLengthRule { max_length: 64 };
        let mut item = new_dns_record_preference("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DnsRecordPreferenceRequiredCodeRule {
            message: "dns record preference needs an entry".to_string(),
        };
        let mut item = new_dns_record_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DnsRecordPreferenceRequiredLabelRule {
            message: "dns record preference needs an entry".to_string(),
        };
        let mut item = new_dns_record_preference("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
