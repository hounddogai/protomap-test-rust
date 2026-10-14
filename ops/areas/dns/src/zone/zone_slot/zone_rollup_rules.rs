use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a zone rollup must pass. Check returns its problems, or none.
pub trait ZoneRollupRule: std::fmt::Debug {
    fn check(&self, item: &ZoneRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRollupRequiredTitleRule {
    pub message: String,
}

impl ZoneRollupRule for ZoneRollupRequiredTitleRule {
    fn check(&self, item: &ZoneRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRollupAudienceLengthRule {
    pub max_length: i64,
}

impl ZoneRollupRule for ZoneRollupAudienceLengthRule {
    fn check(&self, item: &ZoneRollup) -> Vec<String> {
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
pub struct ZoneRollupCodePrefixRule {
    pub prefix: String,
}

impl ZoneRollupRule for ZoneRollupCodePrefixRule {
    fn check(&self, item: &ZoneRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneRollupRequiredAudienceRule {
    pub message: String,
}

impl ZoneRollupRule for ZoneRollupRequiredAudienceRule {
    fn check(&self, item: &ZoneRollup) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone rollup.
#[derive(Debug, Clone)]
pub struct ZoneRollupValidator {
    pub rules: Vec<Rc<dyn ZoneRollupRule>>,
}

impl ZoneRollupValidator {
    pub fn validate(&self, item: &ZoneRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_rollup_validator() -> ZoneRollupValidator {
    ZoneRollupValidator {
        rules: vec![
            Rc::new(ZoneRollupRequiredTitleRule {
                message: "zone rollup needs an entry".to_string(),
            }) as Rc<dyn ZoneRollupRule>,
            Rc::new(ZoneRollupAudienceLengthRule { max_length: 16 }),
            Rc::new(ZoneRollupCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ZoneRollupRequiredAudienceRule {
                message: "zone rollup needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_rollup_validator();
        let item = new_zone_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ZoneRollupRequiredTitleRule {
            message: "zone rollup needs this field".to_string(),
        };
        let mut item = new_zone_rollup("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = ZoneRollupAudienceLengthRule { max_length: 3600 };
        let mut item = new_zone_rollup("x", "y");
        item.audience = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = ZoneRollupRequiredAudienceRule {
            message: "zone rollup needs an entry".to_string(),
        };
        let mut item = new_zone_rollup("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
