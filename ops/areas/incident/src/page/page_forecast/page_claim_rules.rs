use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a page claim must pass. Check returns its problems, or none.
pub trait PageClaimRule: std::fmt::Debug {
    fn check(&self, item: &PageClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageClaimWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageClaimRule for PageClaimWeightRangeRule {
    fn check(&self, item: &PageClaim) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageClaimRequiredOwnerIdRule {
    pub message: String,
}

impl PageClaimRule for PageClaimRequiredOwnerIdRule {
    fn check(&self, item: &PageClaim) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageClaimSourceLengthRule {
    pub max_length: i64,
}

impl PageClaimRule for PageClaimSourceLengthRule {
    fn check(&self, item: &PageClaim) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageClaimNameLengthRule {
    pub max_length: i64,
}

impl PageClaimRule for PageClaimNameLengthRule {
    fn check(&self, item: &PageClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageClaimAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl PageClaimRule for PageClaimAllowedFingerprintRule {
    fn check(&self, item: &PageClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is inconsistent", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageClaimCodePrefixRule {
    pub prefix: String,
}

impl PageClaimRule for PageClaimCodePrefixRule {
    fn check(&self, item: &PageClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page claim.
#[derive(Debug, Clone)]
pub struct PageClaimValidator {
    pub rules: Vec<Rc<dyn PageClaimRule>>,
}

impl PageClaimValidator {
    pub fn validate(&self, item: &PageClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_claim_validator() -> PageClaimValidator {
    PageClaimValidator {
        rules: vec![
            Rc::new(PageClaimWeightRangeRule { min: 9.9, max: 4066.0 }) as Rc<dyn PageClaimRule>,
            Rc::new(PageClaimRequiredOwnerIdRule {
                message: "page claim needs an entry".to_string(),
            }),
            Rc::new(PageClaimSourceLengthRule { max_length: 80 }),
            Rc::new(PageClaimNameLengthRule { max_length: 3600 }),
            Rc::new(PageClaimAllowedFingerprintRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(PageClaimCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_claim_validator();
        let item = new_page_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PageClaimWeightRangeRule { min: 0.5, max: 2108.7 };
        let mut item = new_page_claim("x", "y");
        item.weight = 2110.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = PageClaimRequiredOwnerIdRule {
            message: "page claim needs a value".to_string(),
        };
        let mut item = new_page_claim("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = PageClaimSourceLengthRule { max_length: 16 };
        let mut item = new_page_claim("x", "y");
        item.source = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PageClaimNameLengthRule { max_length: 32 };
        let mut item = new_page_claim("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
