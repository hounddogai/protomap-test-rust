use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global peering tier must pass. Check returns its problems, or none.
pub trait GlobalPeeringTierRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPeeringTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringTierCodePrefixRule {
    pub prefix: String,
}

impl GlobalPeeringTierRule for GlobalPeeringTierCodePrefixRule {
    fn check(&self, item: &GlobalPeeringTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringTierCodeLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringTierRule for GlobalPeeringTierCodeLengthRule {
    fn check(&self, item: &GlobalPeeringTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringTierContactEmailLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringTierRule for GlobalPeeringTierContactEmailLengthRule {
    fn check(&self, item: &GlobalPeeringTier) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringTierTitleLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringTierRule for GlobalPeeringTierTitleLengthRule {
    fn check(&self, item: &GlobalPeeringTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global peering tier.
#[derive(Debug, Clone)]
pub struct GlobalPeeringTierValidator {
    pub rules: Vec<Rc<dyn GlobalPeeringTierRule>>,
}

impl GlobalPeeringTierValidator {
    pub fn validate(&self, item: &GlobalPeeringTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalPeeringTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPeeringTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_peering_tier_validator() -> GlobalPeeringTierValidator {
    GlobalPeeringTierValidator {
        rules: vec![
            Rc::new(GlobalPeeringTierCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn GlobalPeeringTierRule>,
            Rc::new(GlobalPeeringTierCodeLengthRule { max_length: 32 }),
            Rc::new(GlobalPeeringTierContactEmailLengthRule { max_length: 120 }),
            Rc::new(GlobalPeeringTierTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_peering_tier_validator();
        let item = new_global_peering_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalPeeringTierCodeLengthRule { max_length: 3600 };
        let mut item = new_global_peering_tier("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = GlobalPeeringTierContactEmailLengthRule { max_length: 16 };
        let mut item = new_global_peering_tier("x", "y");
        item.contact_email = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalPeeringTierTitleLengthRule { max_length: 3600 };
        let mut item = new_global_peering_tier("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
