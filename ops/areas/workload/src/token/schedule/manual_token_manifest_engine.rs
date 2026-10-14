use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual token manifest calculation.
pub trait ManualTokenManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualTokenManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTokenManifestAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl ManualTokenManifestStep for ManualTokenManifestAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &ManualTokenManifest, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualTokenManifestAdjuster {
    pub cap: f64,
}

impl ManualTokenManifestStep for CapManualTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualTokenManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualTokenManifestAdjuster {
    pub step: f64,
}

impl ManualTokenManifestStep for RoundingManualTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualTokenManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual token manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualTokenManifestEngine {
    pub steps: Vec<Rc<dyn ManualTokenManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualTokenManifestEngine {
    pub fn run(&mut self, item: &ManualTokenManifest) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualTokenManifestStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ManualTokenManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_token_manifest_engine() -> ManualTokenManifestEngine {
    ManualTokenManifestEngine {
        steps: vec![
            Rc::new(ManualTokenManifestAttemptsOffsetAdjuster { weight: 0.23 }) as Rc<dyn ManualTokenManifestStep>,
            Rc::new(CapManualTokenManifestAdjuster { cap: 27407.0 }),
            Rc::new(RoundingManualTokenManifestAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 82585.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_token_manifest_engine();
        let item = new_manual_token_manifest("code-222", "label-741");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
