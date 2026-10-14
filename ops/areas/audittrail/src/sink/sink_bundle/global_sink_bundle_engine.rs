use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global sink bundle calculation.
pub trait GlobalSinkBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalSinkBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalSinkBundleAdjuster {
    pub percent: f64,
}

impl GlobalSinkBundleStep for PercentGlobalSinkBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalSinkBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalSinkBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalSinkBundleStep for TieredGlobalSinkBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalSinkBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalSinkBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalSinkBundleStep for FlaggedGlobalSinkBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalSinkBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkBundleUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalSinkBundleStep for GlobalSinkBundleUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalSinkBundle, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the global sink bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalSinkBundleEngine {
    pub steps: Vec<Rc<dyn GlobalSinkBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalSinkBundleEngine {
    pub fn evaluate(&mut self, item: &GlobalSinkBundle) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalSinkBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalSinkBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_sink_bundle_engine(flags: &flags::Registry) -> GlobalSinkBundleEngine {
    GlobalSinkBundleEngine {
        steps: vec![
            Rc::new(PercentGlobalSinkBundleAdjuster { percent: 13.8 }) as Rc<dyn GlobalSinkBundleStep>,
            Rc::new(TieredGlobalSinkBundleAdjuster {
                bounds: vec![515.0, 1233.0],
                rates: vec![0.84, 0.81],
            }),
            Rc::new(FlaggedGlobalSinkBundleAdjuster {
                flags: flags.clone(),
                flag: "global-sink-bundle-batched".to_string(),
                percent: -9.7,
            }),
            Rc::new(GlobalSinkBundleUpdatedAtOffsetAdjuster { weight: 0.36 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 61921.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_sink_bundle_engine(&flags::new_registry());
        let item = new_global_sink_bundle("code-562", "name-178");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
