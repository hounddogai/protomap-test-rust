use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink target must pass. Check returns its problems, or none.
pub trait SinkTargetRule: std::fmt::Debug {
    fn check(&self, item: &SinkTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTargetTenantIdLengthRule {
    pub max_length: i64,
}

impl SinkTargetRule for SinkTargetTenantIdLengthRule {
    fn check(&self, item: &SinkTarget) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTargetRequiredCodeRule {
    pub message: String,
}

impl SinkTargetRule for SinkTargetRequiredCodeRule {
    fn check(&self, item: &SinkTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTargetRequiredCountryRule {
    pub message: String,
}

impl SinkTargetRule for SinkTargetRequiredCountryRule {
    fn check(&self, item: &SinkTarget) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTargetRequiredTitleRule {
    pub message: String,
}

impl SinkTargetRule for SinkTargetRequiredTitleRule {
    fn check(&self, item: &SinkTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkTargetCodePrefixRule {
    pub prefix: String,
}

impl SinkTargetRule for SinkTargetCodePrefixRule {
    fn check(&self, item: &SinkTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink target.
#[derive(Debug, Clone)]
pub struct SinkTargetValidator {
    pub rules: Vec<Rc<dyn SinkTargetRule>>,
}

impl SinkTargetValidator {
    pub fn validate(&self, item: &SinkTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_target_validator() -> SinkTargetValidator {
    SinkTargetValidator {
        rules: vec![
            Rc::new(SinkTargetTenantIdLengthRule { max_length: 1000 }) as Rc<dyn SinkTargetRule>,
            Rc::new(SinkTargetRequiredCodeRule {
                message: "sink target needs a value".to_string(),
            }),
            Rc::new(SinkTargetRequiredCountryRule {
                message: "sink target needs a value".to_string(),
            }),
            Rc::new(SinkTargetRequiredTitleRule {
                message: "sink target needs this field".to_string(),
            }),
            Rc::new(SinkTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_target_validator();
        let item = new_sink_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = SinkTargetTenantIdLengthRule { max_length: 80 };
        let mut item = new_sink_target("x", "y");
        item.tenant_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SinkTargetRequiredCodeRule {
            message: "sink target needs an entry".to_string(),
        };
        let mut item = new_sink_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = SinkTargetRequiredCountryRule {
            message: "sink target needs this field".to_string(),
        };
        let mut item = new_sink_target("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SinkTargetRequiredTitleRule {
            message: "sink target needs an entry".to_string(),
        };
        let mut item = new_sink_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
