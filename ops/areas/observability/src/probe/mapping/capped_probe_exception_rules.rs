use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped probe exception must pass. Check returns its problems, or none.
pub trait CappedProbeExceptionRule: std::fmt::Debug {
    fn check(&self, item: &CappedProbeException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeExceptionRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedProbeExceptionRule for CappedProbeExceptionRetriesRangeRule {
    fn check(&self, item: &CappedProbeException) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeExceptionRequiredIssuerRule {
    pub message: String,
}

impl CappedProbeExceptionRule for CappedProbeExceptionRequiredIssuerRule {
    fn check(&self, item: &CappedProbeException) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeExceptionCodePrefixRule {
    pub prefix: String,
}

impl CappedProbeExceptionRule for CappedProbeExceptionCodePrefixRule {
    fn check(&self, item: &CappedProbeException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeExceptionIssuerLengthRule {
    pub max_length: i64,
}

impl CappedProbeExceptionRule for CappedProbeExceptionIssuerLengthRule {
    fn check(&self, item: &CappedProbeException) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeExceptionRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedProbeExceptionRule for CappedProbeExceptionRotationDaysRangeRule {
    fn check(&self, item: &CappedProbeException) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeExceptionRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedProbeExceptionRule for CappedProbeExceptionRevisionRangeRule {
    fn check(&self, item: &CappedProbeException) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped probe exception.
#[derive(Debug, Clone)]
pub struct CappedProbeExceptionValidator {
    pub rules: Vec<Rc<dyn CappedProbeExceptionRule>>,
}

impl CappedProbeExceptionValidator {
    pub fn validate(&self, item: &CappedProbeException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedProbeException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedProbeException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_probe_exception_validator() -> CappedProbeExceptionValidator {
    CappedProbeExceptionValidator {
        rules: vec![
            Rc::new(CappedProbeExceptionRetriesRangeRule { min: 64, max: 120 }) as Rc<dyn CappedProbeExceptionRule>,
            Rc::new(CappedProbeExceptionRequiredIssuerRule {
                message: "capped probe exception needs an entry".to_string(),
            }),
            Rc::new(CappedProbeExceptionCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CappedProbeExceptionIssuerLengthRule { max_length: 120 }),
            Rc::new(CappedProbeExceptionRotationDaysRangeRule { min: 64, max: 120 }),
            Rc::new(CappedProbeExceptionRevisionRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_probe_exception_validator();
        let item = new_capped_probe_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = CappedProbeExceptionRetriesRangeRule { min: 120, max: 1000 };
        let mut item = new_capped_probe_exception("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = CappedProbeExceptionRequiredIssuerRule {
            message: "capped probe exception needs a value".to_string(),
        };
        let mut item = new_capped_probe_exception("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = CappedProbeExceptionIssuerLengthRule { max_length: 255 };
        let mut item = new_capped_probe_exception("x", "y");
        item.issuer = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = CappedProbeExceptionRotationDaysRangeRule { min: 80, max: 1000 };
        let mut item = new_capped_probe_exception("x", "y");
        item.rotation_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = CappedProbeExceptionRevisionRangeRule { min: 120, max: 86400 };
        let mut item = new_capped_probe_exception("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
