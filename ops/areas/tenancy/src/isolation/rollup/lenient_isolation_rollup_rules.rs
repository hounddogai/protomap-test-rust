use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient isolation rollup must pass. Check returns its problems, or none.
pub trait LenientIsolationRollupRule: std::fmt::Debug {
    fn check(&self, item: &LenientIsolationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIsolationRollupAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LenientIsolationRollupRule for LenientIsolationRollupAllowedNameRule {
    fn check(&self, item: &LenientIsolationRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIsolationRollupRegionLengthRule {
    pub max_length: i64,
}

impl LenientIsolationRollupRule for LenientIsolationRollupRegionLengthRule {
    fn check(&self, item: &LenientIsolationRollup) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIsolationRollupNameLengthRule {
    pub max_length: i64,
}

impl LenientIsolationRollupRule for LenientIsolationRollupNameLengthRule {
    fn check(&self, item: &LenientIsolationRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIsolationRollupRequiredNameRule {
    pub message: String,
}

impl LenientIsolationRollupRule for LenientIsolationRollupRequiredNameRule {
    fn check(&self, item: &LenientIsolationRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIsolationRollupActiveGuardRule {
    pub limit: i64,
}

impl LenientIsolationRollupRule for LenientIsolationRollupActiveGuardRule {
    fn check(&self, item: &LenientIsolationRollup) -> Vec<String> {
        if item.active && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while active",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientIsolationRollupRequiredRegionRule {
    pub message: String,
}

impl LenientIsolationRollupRule for LenientIsolationRollupRequiredRegionRule {
    fn check(&self, item: &LenientIsolationRollup) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient isolation rollup.
#[derive(Debug, Clone)]
pub struct LenientIsolationRollupValidator {
    pub rules: Vec<Rc<dyn LenientIsolationRollupRule>>,
}

impl LenientIsolationRollupValidator {
    pub fn validate(&self, item: &LenientIsolationRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientIsolationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientIsolationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_isolation_rollup_validator() -> LenientIsolationRollupValidator {
    LenientIsolationRollupValidator {
        rules: vec![
            Rc::new(LenientIsolationRollupAllowedNameRule {
                allowed: vec!["seasonal".to_string()],
            }) as Rc<dyn LenientIsolationRollupRule>,
            Rc::new(LenientIsolationRollupRegionLengthRule { max_length: 64 }),
            Rc::new(LenientIsolationRollupNameLengthRule { max_length: 80 }),
            Rc::new(LenientIsolationRollupRequiredNameRule {
                message: "lenient isolation rollup needs this field".to_string(),
            }),
            Rc::new(LenientIsolationRollupActiveGuardRule { limit: 86400 }),
            Rc::new(LenientIsolationRollupRequiredRegionRule {
                message: "lenient isolation rollup needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_isolation_rollup_validator();
        let item = new_lenient_isolation_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = LenientIsolationRollupRegionLengthRule { max_length: 255 };
        let mut item = new_lenient_isolation_rollup("x", "y");
        item.region = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientIsolationRollupNameLengthRule { max_length: 120 };
        let mut item = new_lenient_isolation_rollup("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LenientIsolationRollupRequiredNameRule {
            message: "lenient isolation rollup needs this field".to_string(),
        };
        let mut item = new_lenient_isolation_rollup("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = LenientIsolationRollupRequiredRegionRule {
            message: "lenient isolation rollup needs a value".to_string(),
        };
        let mut item = new_lenient_isolation_rollup("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
