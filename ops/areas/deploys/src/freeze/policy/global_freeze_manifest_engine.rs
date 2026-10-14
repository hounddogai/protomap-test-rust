use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global freeze manifest calculation.
pub trait GlobalFreezeManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalFreezeManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalFreezeManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalFreezeManifestFactor for TieredGlobalFreezeManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalFreezeManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalFreezeManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalFreezeManifestFactor for FlaggedGlobalFreezeManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalFreezeManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalFreezeManifestAdjuster {
    pub step: f64,
}

impl GlobalFreezeManifestFactor for RoundingGlobalFreezeManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalFreezeManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global freeze manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalFreezeManifestEngine {
    pub steps: Vec<Rc<dyn GlobalFreezeManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalFreezeManifestEngine {
    pub fn evaluate(&mut self, item: &GlobalFreezeManifest) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[GlobalFreezeManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_freeze_manifest_engine(flags: &flags::Registry) -> GlobalFreezeManifestEngine {
    GlobalFreezeManifestEngine {
        steps: vec![
            Rc::new(TieredGlobalFreezeManifestAdjuster {
                bounds: vec![1353.0, 2570.0, 2852.0],
                rates: vec![0.96, 0.86, 1.07],
            }) as Rc<dyn GlobalFreezeManifestFactor>,
            Rc::new(FlaggedGlobalFreezeManifestAdjuster {
                flags: flags.clone(),
                flag: "global-freeze-manifest-seasonal".to_string(),
                percent: -9.2,
            }),
            Rc::new(RoundingGlobalFreezeManifestAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 48875.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_freeze_manifest_engine(&flags::new_registry());
        let item = new_global_freeze_manifest("code-388", "title-915");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
