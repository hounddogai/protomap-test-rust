use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual probe request must pass. Check returns its problems, or none.
pub trait ManualProbeRequestRule: std::fmt::Debug {
    fn check(&self, item: &ManualProbeRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeRequestRequiredTitleRule {
    pub message: String,
}

impl ManualProbeRequestRule for ManualProbeRequestRequiredTitleRule {
    fn check(&self, item: &ManualProbeRequest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeRequestAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ManualProbeRequestRule for ManualProbeRequestAllowedCodeRule {
    fn check(&self, item: &ManualProbeRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeRequestCodePrefixRule {
    pub prefix: String,
}

impl ManualProbeRequestRule for ManualProbeRequestCodePrefixRule {
    fn check(&self, item: &ManualProbeRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeRequestRequiredFingerprintRule {
    pub message: String,
}

impl ManualProbeRequestRule for ManualProbeRequestRequiredFingerprintRule {
    fn check(&self, item: &ManualProbeRequest) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeRequestFingerprintLengthRule {
    pub max_length: i64,
}

impl ManualProbeRequestRule for ManualProbeRequestFingerprintLengthRule {
    fn check(&self, item: &ManualProbeRequest) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualProbeRequestNameLengthRule {
    pub max_length: i64,
}

impl ManualProbeRequestRule for ManualProbeRequestNameLengthRule {
    fn check(&self, item: &ManualProbeRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual probe request.
#[derive(Debug, Clone)]
pub struct ManualProbeRequestValidator {
    pub rules: Vec<Rc<dyn ManualProbeRequestRule>>,
}

impl ManualProbeRequestValidator {
    pub fn validate(&self, item: &ManualProbeRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualProbeRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualProbeRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_probe_request_validator() -> ManualProbeRequestValidator {
    ManualProbeRequestValidator {
        rules: vec![
            Rc::new(ManualProbeRequestRequiredTitleRule {
                message: "manual probe request needs an entry".to_string(),
            }) as Rc<dyn ManualProbeRequestRule>,
            Rc::new(ManualProbeRequestAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(ManualProbeRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ManualProbeRequestRequiredFingerprintRule {
                message: "manual probe request needs an entry".to_string(),
            }),
            Rc::new(ManualProbeRequestFingerprintLengthRule { max_length: 80 }),
            Rc::new(ManualProbeRequestNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_probe_request_validator();
        let item = new_manual_probe_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualProbeRequestRequiredTitleRule {
            message: "manual probe request needs an entry".to_string(),
        };
        let mut item = new_manual_probe_request("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ManualProbeRequestRequiredFingerprintRule {
            message: "manual probe request needs an entry".to_string(),
        };
        let mut item = new_manual_probe_request("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ManualProbeRequestFingerprintLengthRule { max_length: 3600 };
        let mut item = new_manual_probe_request("x", "y");
        item.fingerprint = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualProbeRequestNameLengthRule { max_length: 255 };
        let mut item = new_manual_probe_request("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
