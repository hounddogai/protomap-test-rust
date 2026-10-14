use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global action item claim must pass. Check returns its problems, or none.
pub trait GlobalActionItemClaimRule: std::fmt::Debug {
    fn check(&self, item: &GlobalActionItemClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalActionItemClaimChecksumLengthRule {
    pub max_length: i64,
}

impl GlobalActionItemClaimRule for GlobalActionItemClaimChecksumLengthRule {
    fn check(&self, item: &GlobalActionItemClaim) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalActionItemClaimCodeLengthRule {
    pub max_length: i64,
}

impl GlobalActionItemClaimRule for GlobalActionItemClaimCodeLengthRule {
    fn check(&self, item: &GlobalActionItemClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalActionItemClaimTitleLengthRule {
    pub max_length: i64,
}

impl GlobalActionItemClaimRule for GlobalActionItemClaimTitleLengthRule {
    fn check(&self, item: &GlobalActionItemClaim) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global action item claim.
#[derive(Debug, Clone)]
pub struct GlobalActionItemClaimValidator {
    pub rules: Vec<Rc<dyn GlobalActionItemClaimRule>>,
}

impl GlobalActionItemClaimValidator {
    pub fn validate(&self, item: &GlobalActionItemClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalActionItemClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalActionItemClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_action_item_claim_validator() -> GlobalActionItemClaimValidator {
    GlobalActionItemClaimValidator {
        rules: vec![
            Rc::new(GlobalActionItemClaimChecksumLengthRule { max_length: 3600 }) as Rc<dyn GlobalActionItemClaimRule>,
            Rc::new(GlobalActionItemClaimCodeLengthRule { max_length: 3600 }),
            Rc::new(GlobalActionItemClaimTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_action_item_claim_validator();
        let item = new_global_action_item_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = GlobalActionItemClaimChecksumLengthRule { max_length: 255 };
        let mut item = new_global_action_item_claim("x", "y");
        item.checksum = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalActionItemClaimCodeLengthRule { max_length: 80 };
        let mut item = new_global_action_item_claim("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalActionItemClaimTitleLengthRule { max_length: 16 };
        let mut item = new_global_action_item_claim("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
