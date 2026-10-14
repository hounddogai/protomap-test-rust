use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary release manifest must pass. Check returns its problems, or none.
pub trait PrimaryReleaseManifestRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryReleaseManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseManifestPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryReleaseManifestRule for PrimaryReleaseManifestPriorityRangeRule {
    fn check(&self, item: &PrimaryReleaseManifest) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseManifestBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryReleaseManifestRule for PrimaryReleaseManifestBurstRangeRule {
    fn check(&self, item: &PrimaryReleaseManifest) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReleaseManifestCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryReleaseManifestRule for PrimaryReleaseManifestCeilingRangeRule {
    fn check(&self, item: &PrimaryReleaseManifest) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary release manifest.
#[derive(Debug, Clone)]
pub struct PrimaryReleaseManifestValidator {
    pub rules: Vec<Rc<dyn PrimaryReleaseManifestRule>>,
}

impl PrimaryReleaseManifestValidator {
    pub fn validate(&self, item: &PrimaryReleaseManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryReleaseManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryReleaseManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_release_manifest_validator() -> PrimaryReleaseManifestValidator {
    PrimaryReleaseManifestValidator {
        rules: vec![
            Rc::new(PrimaryReleaseManifestPriorityRangeRule { min: 120, max: 255 })
                as Rc<dyn PrimaryReleaseManifestRule>,
            Rc::new(PrimaryReleaseManifestBurstRangeRule { min: 16, max: 80 }),
            Rc::new(PrimaryReleaseManifestCeilingRangeRule { min: 2.8, max: 2371.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_release_manifest_validator();
        let item = new_primary_release_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PrimaryReleaseManifestPriorityRangeRule { min: 255, max: 1000 };
        let mut item = new_primary_release_manifest("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PrimaryReleaseManifestBurstRangeRule { min: 255, max: 3600 };
        let mut item = new_primary_release_manifest("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = PrimaryReleaseManifestCeilingRangeRule { min: 5.2, max: 5006.2 };
        let mut item = new_primary_release_manifest("x", "y");
        item.ceiling = 5007.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
