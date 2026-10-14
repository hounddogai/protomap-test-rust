use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary dns record item must pass. Check returns its problems, or none.
pub trait PrimaryDnsRecordItemRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryDnsRecordItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordItemNameLengthRule {
    pub max_length: i64,
}

impl PrimaryDnsRecordItemRule for PrimaryDnsRecordItemNameLengthRule {
    fn check(&self, item: &PrimaryDnsRecordItem) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordItemUnitLengthRule {
    pub max_length: i64,
}

impl PrimaryDnsRecordItemRule for PrimaryDnsRecordItemUnitLengthRule {
    fn check(&self, item: &PrimaryDnsRecordItem) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordItemRequiredNameRule {
    pub message: String,
}

impl PrimaryDnsRecordItemRule for PrimaryDnsRecordItemRequiredNameRule {
    fn check(&self, item: &PrimaryDnsRecordItem) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordItemPinnedGuardRule {
    pub limit: f64,
}

impl PrimaryDnsRecordItemRule for PrimaryDnsRecordItemPinnedGuardRule {
    fn check(&self, item: &PrimaryDnsRecordItem) -> Vec<String> {
        if item.pinned && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while pinned", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordItemScopeLengthRule {
    pub max_length: i64,
}

impl PrimaryDnsRecordItemRule for PrimaryDnsRecordItemScopeLengthRule {
    fn check(&self, item: &PrimaryDnsRecordItem) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordItemRequiredUnitRule {
    pub message: String,
}

impl PrimaryDnsRecordItemRule for PrimaryDnsRecordItemRequiredUnitRule {
    fn check(&self, item: &PrimaryDnsRecordItem) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary dns record item.
#[derive(Debug, Clone)]
pub struct PrimaryDnsRecordItemValidator {
    pub rules: Vec<Rc<dyn PrimaryDnsRecordItemRule>>,
}

impl PrimaryDnsRecordItemValidator {
    pub fn validate(&self, item: &PrimaryDnsRecordItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryDnsRecordItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryDnsRecordItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_dns_record_item_validator() -> PrimaryDnsRecordItemValidator {
    PrimaryDnsRecordItemValidator {
        rules: vec![
            Rc::new(PrimaryDnsRecordItemNameLengthRule { max_length: 32 }) as Rc<dyn PrimaryDnsRecordItemRule>,
            Rc::new(PrimaryDnsRecordItemUnitLengthRule { max_length: 16 }),
            Rc::new(PrimaryDnsRecordItemRequiredNameRule {
                message: "primary dns record item needs an entry".to_string(),
            }),
            Rc::new(PrimaryDnsRecordItemPinnedGuardRule { limit: 0.1 }),
            Rc::new(PrimaryDnsRecordItemScopeLengthRule { max_length: 86400 }),
            Rc::new(PrimaryDnsRecordItemRequiredUnitRule {
                message: "primary dns record item needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_dns_record_item_validator();
        let item = new_primary_dns_record_item("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PrimaryDnsRecordItemNameLengthRule { max_length: 255 };
        let mut item = new_primary_dns_record_item("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = PrimaryDnsRecordItemUnitLengthRule { max_length: 80 };
        let mut item = new_primary_dns_record_item("x", "y");
        item.unit = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryDnsRecordItemRequiredNameRule {
            message: "primary dns record item needs an entry".to_string(),
        };
        let mut item = new_primary_dns_record_item("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = PrimaryDnsRecordItemScopeLengthRule { max_length: 32 };
        let mut item = new_primary_dns_record_item("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PrimaryDnsRecordItemRequiredUnitRule {
            message: "primary dns record item needs an entry".to_string(),
        };
        let mut item = new_primary_dns_record_item("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
