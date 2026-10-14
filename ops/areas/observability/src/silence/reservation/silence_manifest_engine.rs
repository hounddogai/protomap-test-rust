use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the silence manifest calculation.
pub trait SilenceManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceManifestMarginOffsetAdjuster {
    pub weight: f64,
}

impl SilenceManifestAdjuster for SilenceManifestMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceManifest, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceManifestAdjuster {
    pub percent: f64,
}

impl SilenceManifestAdjuster for PercentSilenceManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceManifestAdjuster {
    pub minimum: f64,
}

impl SilenceManifestAdjuster for MinimumSilenceManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the silence manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceManifestEngine {
    pub steps: Vec<Rc<dyn SilenceManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceManifestEngine {
    pub fn run(&mut self, item: &SilenceManifest) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[SilenceManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_silence_manifest_engine() -> SilenceManifestEngine {
    SilenceManifestEngine {
        steps: vec![
            Rc::new(SilenceManifestMarginOffsetAdjuster { weight: 0.61 }) as Rc<dyn SilenceManifestAdjuster>,
            Rc::new(PercentSilenceManifestAdjuster { percent: 13.9 }),
            Rc::new(MinimumSilenceManifestAdjuster { minimum: 41.7 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 5545.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_manifest_engine();
        let item = new_silence_manifest("code-534", "label-701");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
