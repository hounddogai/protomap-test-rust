use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered query manifest calculation.
pub trait TieredQueryManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredQueryManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredQueryManifestAdjuster {
    pub step: f64,
}

impl TieredQueryManifestFactor for RoundingTieredQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredQueryManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredQueryManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredQueryManifestFactor for FlaggedTieredQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredQueryManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredQueryManifestAdjuster {
    pub percent: f64,
}

impl TieredQueryManifestFactor for PercentTieredQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredQueryManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered query manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredQueryManifestEngine {
    pub steps: Vec<Rc<dyn TieredQueryManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredQueryManifestEngine {
    pub fn evaluate(&mut self, item: &TieredQueryManifest) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredQueryManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredQueryManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_query_manifest_engine(flags: &flags::Registry) -> TieredQueryManifestEngine {
    TieredQueryManifestEngine {
        steps: vec![
            Rc::new(RoundingTieredQueryManifestAdjuster { step: 0.05 }) as Rc<dyn TieredQueryManifestFactor>,
            Rc::new(FlaggedTieredQueryManifestAdjuster {
                flags: flags.clone(),
                flag: "tiered-query-manifest-weighted".to_string(),
                percent: -2.7,
            }),
            Rc::new(PercentTieredQueryManifestAdjuster { percent: -3.1 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 57341.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_query_manifest_engine(&flags::new_registry());
        let item = new_tiered_query_manifest("code-659", "title-183");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
