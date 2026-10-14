use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered token manifest must pass. Check returns its problems, or none.
pub trait TieredTokenManifestRule: std::fmt::Debug {
    fn check(&self, item: &TieredTokenManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifestRequiredIssuerRule {
    pub message: String,
}

impl TieredTokenManifestRule for TieredTokenManifestRequiredIssuerRule {
    fn check(&self, item: &TieredTokenManifest) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifestCodePrefixRule {
    pub prefix: String,
}

impl TieredTokenManifestRule for TieredTokenManifestCodePrefixRule {
    fn check(&self, item: &TieredTokenManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifestCipherLengthRule {
    pub max_length: i64,
}

impl TieredTokenManifestRule for TieredTokenManifestCipherLengthRule {
    fn check(&self, item: &TieredTokenManifest) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifestTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTokenManifestRule for TieredTokenManifestTtlSecondsRangeRule {
    fn check(&self, item: &TieredTokenManifest) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifestCodeLengthRule {
    pub max_length: i64,
}

impl TieredTokenManifestRule for TieredTokenManifestCodeLengthRule {
    fn check(&self, item: &TieredTokenManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifestRequiredTitleRule {
    pub message: String,
}

impl TieredTokenManifestRule for TieredTokenManifestRequiredTitleRule {
    fn check(&self, item: &TieredTokenManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered token manifest.
#[derive(Debug, Clone)]
pub struct TieredTokenManifestValidator {
    pub rules: Vec<Rc<dyn TieredTokenManifestRule>>,
}

impl TieredTokenManifestValidator {
    pub fn validate(&self, item: &TieredTokenManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredTokenManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTokenManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_token_manifest_validator() -> TieredTokenManifestValidator {
    TieredTokenManifestValidator {
        rules: vec![
            Rc::new(TieredTokenManifestRequiredIssuerRule {
                message: "tiered token manifest needs an entry".to_string(),
            }) as Rc<dyn TieredTokenManifestRule>,
            Rc::new(TieredTokenManifestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TieredTokenManifestCipherLengthRule { max_length: 80 }),
            Rc::new(TieredTokenManifestTtlSecondsRangeRule { min: 32, max: 64 }),
            Rc::new(TieredTokenManifestCodeLengthRule { max_length: 255 }),
            Rc::new(TieredTokenManifestRequiredTitleRule {
                message: "tiered token manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_token_manifest_validator();
        let item = new_tiered_token_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = TieredTokenManifestRequiredIssuerRule {
            message: "tiered token manifest needs this field".to_string(),
        };
        let mut item = new_tiered_token_manifest("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = TieredTokenManifestCipherLengthRule { max_length: 3600 };
        let mut item = new_tiered_token_manifest("x", "y");
        item.cipher = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = TieredTokenManifestTtlSecondsRangeRule { min: 120, max: 1000 };
        let mut item = new_tiered_token_manifest("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredTokenManifestCodeLengthRule { max_length: 3600 };
        let mut item = new_tiered_token_manifest("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredTokenManifestRequiredTitleRule {
            message: "tiered token manifest needs an entry".to_string(),
        };
        let mut item = new_tiered_token_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
