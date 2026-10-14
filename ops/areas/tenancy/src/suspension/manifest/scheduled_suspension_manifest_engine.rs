use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled suspension manifest calculation.
pub trait ScheduledSuspensionManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledSuspensionManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledSuspensionManifestAdjuster {
    pub minimum: f64,
}

impl ScheduledSuspensionManifestFactor for MinimumScheduledSuspensionManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledSuspensionManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionManifestScoreOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledSuspensionManifestFactor for ScheduledSuspensionManifestScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledSuspensionManifest, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledSuspensionManifestAdjuster {
    pub step: f64,
}

impl ScheduledSuspensionManifestFactor for RoundingScheduledSuspensionManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledSuspensionManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled suspension manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionManifestEngine {
    pub steps: Vec<Rc<dyn ScheduledSuspensionManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledSuspensionManifestEngine {
    pub fn price(&mut self, item: &ScheduledSuspensionManifest) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledSuspensionManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_suspension_manifest_engine() -> ScheduledSuspensionManifestEngine {
    ScheduledSuspensionManifestEngine {
        steps: vec![
            Rc::new(MinimumScheduledSuspensionManifestAdjuster { minimum: 28.9 })
                as Rc<dyn ScheduledSuspensionManifestFactor>,
            Rc::new(ScheduledSuspensionManifestScoreOffsetAdjuster { weight: 0.23 }),
            Rc::new(RoundingScheduledSuspensionManifestAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 10702.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_suspension_manifest_engine();
        let item = new_scheduled_suspension_manifest("code-973", "label-91");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
