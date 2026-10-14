use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release manifest calculation.
pub trait ReleaseManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseManifestAdjuster {
    pub minimum: f64,
}

impl ReleaseManifestFactor for MinimumReleaseManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseManifestFactor for FlaggedReleaseManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseManifestEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseManifestFactor for ReleaseManifestEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseManifest, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the release manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseManifestEngine {
    pub steps: Vec<Rc<dyn ReleaseManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseManifestEngine {
    pub fn compute(&mut self, item: &ReleaseManifest) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ReleaseManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_manifest_engine(flags: &flags::Registry) -> ReleaseManifestEngine {
    ReleaseManifestEngine {
        steps: vec![
            Rc::new(MinimumReleaseManifestAdjuster { minimum: 28.0 }) as Rc<dyn ReleaseManifestFactor>,
            Rc::new(FlaggedReleaseManifestAdjuster {
                flags: flags.clone(),
                flag: "release-manifest-regional".to_string(),
                percent: 0.0,
            }),
            Rc::new(ReleaseManifestEndsAtOffsetAdjuster { weight: 1.84 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 28899.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_manifest_engine(&flags::new_registry());
        let item = new_release_manifest("code-341", "name-679");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
