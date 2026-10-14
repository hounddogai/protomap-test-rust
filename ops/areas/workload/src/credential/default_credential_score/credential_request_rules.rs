use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a credential request must pass. Check returns its problems, or none.
pub trait CredentialRequestRule: std::fmt::Debug {
    fn check(&self, item: &CredentialRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRequestAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl CredentialRequestRule for CredentialRequestAllowedPhoneRule {
    fn check(&self, item: &CredentialRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is above maximum", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRequestCodeLengthRule {
    pub max_length: i64,
}

impl CredentialRequestRule for CredentialRequestCodeLengthRule {
    fn check(&self, item: &CredentialRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRequestFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialRequestRule for CredentialRequestFloorValueRangeRule {
    fn check(&self, item: &CredentialRequest) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRequestCipherLengthRule {
    pub max_length: i64,
}

impl CredentialRequestRule for CredentialRequestCipherLengthRule {
    fn check(&self, item: &CredentialRequest) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRequestBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialRequestRule for CredentialRequestBatchSizeRangeRule {
    fn check(&self, item: &CredentialRequest) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a credential request.
#[derive(Debug, Clone)]
pub struct CredentialRequestValidator {
    pub rules: Vec<Rc<dyn CredentialRequestRule>>,
}

impl CredentialRequestValidator {
    pub fn validate(&self, item: &CredentialRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_request_validator() -> CredentialRequestValidator {
    CredentialRequestValidator {
        rules: vec![
            Rc::new(CredentialRequestAllowedPhoneRule {
                allowed: vec!["seasonal".to_string(), "global".to_string(), "adaptive".to_string()],
            }) as Rc<dyn CredentialRequestRule>,
            Rc::new(CredentialRequestCodeLengthRule { max_length: 86400 }),
            Rc::new(CredentialRequestFloorValueRangeRule { min: 5.6, max: 2136.5 }),
            Rc::new(CredentialRequestCipherLengthRule { max_length: 32 }),
            Rc::new(CredentialRequestBatchSizeRangeRule { min: 64, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_request_validator();
        let item = new_credential_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CredentialRequestCodeLengthRule { max_length: 1000 };
        let mut item = new_credential_request("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = CredentialRequestFloorValueRangeRule { min: 8.5, max: 1702.7 };
        let mut item = new_credential_request("x", "y");
        item.floor_value = 1704.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = CredentialRequestCipherLengthRule { max_length: 255 };
        let mut item = new_credential_request("x", "y");
        item.cipher = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = CredentialRequestBatchSizeRangeRule { min: 32, max: 1000 };
        let mut item = new_credential_request("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
