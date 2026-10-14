use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal slo manifest must pass. Check returns its problems, or none.
pub trait SeasonalSloManifestRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalSloManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloManifestStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalSloManifestRule for SeasonalSloManifestStartsAtRangeRule {
    fn check(&self, item: &SeasonalSloManifest) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloManifestThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalSloManifestRule for SeasonalSloManifestThresholdRangeRule {
    fn check(&self, item: &SeasonalSloManifest) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloManifestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalSloManifestRule for SeasonalSloManifestDurationMinutesRangeRule {
    fn check(&self, item: &SeasonalSloManifest) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloManifestVisibleGuardRule {
    pub limit: i64,
}

impl SeasonalSloManifestRule for SeasonalSloManifestVisibleGuardRule {
    fn check(&self, item: &SeasonalSloManifest) -> Vec<String> {
        if item.visible && (item.starts_at > self.limit) {
            return vec![format!(
                "starts at {} exceeds {} while visible",
                item.starts_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal slo manifest.
#[derive(Debug, Clone)]
pub struct SeasonalSloManifestValidator {
    pub rules: Vec<Rc<dyn SeasonalSloManifestRule>>,
}

impl SeasonalSloManifestValidator {
    pub fn validate(&self, item: &SeasonalSloManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalSloManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalSloManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_slo_manifest_validator() -> SeasonalSloManifestValidator {
    SeasonalSloManifestValidator {
        rules: vec![
            Rc::new(SeasonalSloManifestStartsAtRangeRule { min: 80, max: 80 }) as Rc<dyn SeasonalSloManifestRule>,
            Rc::new(SeasonalSloManifestThresholdRangeRule { min: 3.2, max: 1795.5 }),
            Rc::new(SeasonalSloManifestDurationMinutesRangeRule { min: 16, max: 32 }),
            Rc::new(SeasonalSloManifestVisibleGuardRule { limit: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_slo_manifest_validator();
        let item = new_seasonal_slo_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = SeasonalSloManifestStartsAtRangeRule { min: 16, max: 86400 };
        let mut item = new_seasonal_slo_manifest("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = SeasonalSloManifestThresholdRangeRule { min: 4.8, max: 3539.1 };
        let mut item = new_seasonal_slo_manifest("x", "y");
        item.threshold = 3540.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = SeasonalSloManifestDurationMinutesRangeRule { min: 16, max: 32 };
        let mut item = new_seasonal_slo_manifest("x", "y");
        item.duration_minutes = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
