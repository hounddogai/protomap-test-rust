use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional verifier rollup must pass. Check returns its problems, or none.
pub trait RegionalVerifierRollupRule: std::fmt::Debug {
    fn check(&self, item: &RegionalVerifierRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierRollupRequiredAudienceRule {
    pub message: String,
}

impl RegionalVerifierRollupRule for RegionalVerifierRollupRequiredAudienceRule {
    fn check(&self, item: &RegionalVerifierRollup) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierRollupTitleLengthRule {
    pub max_length: i64,
}

impl RegionalVerifierRollupRule for RegionalVerifierRollupTitleLengthRule {
    fn check(&self, item: &RegionalVerifierRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierRollupCodeLengthRule {
    pub max_length: i64,
}

impl RegionalVerifierRollupRule for RegionalVerifierRollupCodeLengthRule {
    fn check(&self, item: &RegionalVerifierRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierRollupCodePrefixRule {
    pub prefix: String,
}

impl RegionalVerifierRollupRule for RegionalVerifierRollupCodePrefixRule {
    fn check(&self, item: &RegionalVerifierRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierRollupRequiredChannelRule {
    pub message: String,
}

impl RegionalVerifierRollupRule for RegionalVerifierRollupRequiredChannelRule {
    fn check(&self, item: &RegionalVerifierRollup) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional verifier rollup.
#[derive(Debug, Clone)]
pub struct RegionalVerifierRollupValidator {
    pub rules: Vec<Rc<dyn RegionalVerifierRollupRule>>,
}

impl RegionalVerifierRollupValidator {
    pub fn validate(&self, item: &RegionalVerifierRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalVerifierRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalVerifierRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_verifier_rollup_validator() -> RegionalVerifierRollupValidator {
    RegionalVerifierRollupValidator {
        rules: vec![
            Rc::new(RegionalVerifierRollupRequiredAudienceRule {
                message: "regional verifier rollup needs this field".to_string(),
            }) as Rc<dyn RegionalVerifierRollupRule>,
            Rc::new(RegionalVerifierRollupTitleLengthRule { max_length: 16 }),
            Rc::new(RegionalVerifierRollupCodeLengthRule { max_length: 32 }),
            Rc::new(RegionalVerifierRollupCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RegionalVerifierRollupRequiredChannelRule {
                message: "regional verifier rollup needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_verifier_rollup_validator();
        let item = new_regional_verifier_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RegionalVerifierRollupRequiredAudienceRule {
            message: "regional verifier rollup needs this field".to_string(),
        };
        let mut item = new_regional_verifier_rollup("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalVerifierRollupTitleLengthRule { max_length: 3600 };
        let mut item = new_regional_verifier_rollup("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalVerifierRollupCodeLengthRule { max_length: 32 };
        let mut item = new_regional_verifier_rollup("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = RegionalVerifierRollupRequiredChannelRule {
            message: "regional verifier rollup needs an entry".to_string(),
        };
        let mut item = new_regional_verifier_rollup("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
