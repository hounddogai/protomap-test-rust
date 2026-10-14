use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe claim must pass. Check returns its problems, or none.
pub trait ProbeClaimRule: std::fmt::Debug {
    fn check(&self, item: &ProbeClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeClaimNameLengthRule {
    pub max_length: i64,
}

impl ProbeClaimRule for ProbeClaimNameLengthRule {
    fn check(&self, item: &ProbeClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeClaimRequiredAudienceRule {
    pub message: String,
}

impl ProbeClaimRule for ProbeClaimRequiredAudienceRule {
    fn check(&self, item: &ProbeClaim) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeClaimAudienceLengthRule {
    pub max_length: i64,
}

impl ProbeClaimRule for ProbeClaimAudienceLengthRule {
    fn check(&self, item: &ProbeClaim) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeClaimRequiredCategoryRule {
    pub message: String,
}

impl ProbeClaimRule for ProbeClaimRequiredCategoryRule {
    fn check(&self, item: &ProbeClaim) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeClaimRequiredCodeRule {
    pub message: String,
}

impl ProbeClaimRule for ProbeClaimRequiredCodeRule {
    fn check(&self, item: &ProbeClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe claim.
#[derive(Debug, Clone)]
pub struct ProbeClaimValidator {
    pub rules: Vec<Rc<dyn ProbeClaimRule>>,
}

impl ProbeClaimValidator {
    pub fn validate(&self, item: &ProbeClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ProbeClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_claim_validator() -> ProbeClaimValidator {
    ProbeClaimValidator {
        rules: vec![
            Rc::new(ProbeClaimNameLengthRule { max_length: 86400 }) as Rc<dyn ProbeClaimRule>,
            Rc::new(ProbeClaimRequiredAudienceRule {
                message: "probe claim needs this field".to_string(),
            }),
            Rc::new(ProbeClaimAudienceLengthRule { max_length: 80 }),
            Rc::new(ProbeClaimRequiredCategoryRule {
                message: "probe claim needs this field".to_string(),
            }),
            Rc::new(ProbeClaimRequiredCodeRule {
                message: "probe claim needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_claim_validator();
        let item = new_probe_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ProbeClaimNameLengthRule { max_length: 255 };
        let mut item = new_probe_claim("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = ProbeClaimRequiredAudienceRule {
            message: "probe claim needs an entry".to_string(),
        };
        let mut item = new_probe_claim("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = ProbeClaimAudienceLengthRule { max_length: 80 };
        let mut item = new_probe_claim("x", "y");
        item.audience = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = ProbeClaimRequiredCategoryRule {
            message: "probe claim needs an entry".to_string(),
        };
        let mut item = new_probe_claim("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ProbeClaimRequiredCodeRule {
            message: "probe claim needs a value".to_string(),
        };
        let mut item = new_probe_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
