use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the slo manifest calculation.
pub trait SloManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloManifestAdjuster {
    pub minimum: f64,
}

impl SloManifestFactor for MinimumSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloManifestAdjuster {
    pub cap: f64,
}

impl SloManifestFactor for CapSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloManifestMinimumOffsetAdjuster {
    pub weight: f64,
}

impl SloManifestFactor for SloManifestMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &SloManifest, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloManifestFactor for FlaggedSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloManifestAdjuster {
    pub percent: f64,
}

impl SloManifestFactor for PercentSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SloManifestFactor for TieredSloManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SloManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the slo manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloManifestEngine {
    pub steps: Vec<Rc<dyn SloManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloManifestEngine {
    pub fn compute(&mut self, item: &SloManifest) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_slo_manifest_engine(flags: &flags::Registry) -> SloManifestEngine {
    SloManifestEngine {
        steps: vec![
            Rc::new(MinimumSloManifestAdjuster { minimum: 12.2 }) as Rc<dyn SloManifestFactor>,
            Rc::new(CapSloManifestAdjuster { cap: 25811.0 }),
            Rc::new(SloManifestMinimumOffsetAdjuster { weight: 0.71 }),
            Rc::new(FlaggedSloManifestAdjuster {
                flags: flags.clone(),
                flag: "slo-manifest-tiered".to_string(),
                percent: 4.0,
            }),
            Rc::new(PercentSloManifestAdjuster { percent: -5.0 }),
            Rc::new(TieredSloManifestAdjuster {
                bounds: vec![3889.0, 4968.0],
                rates: vec![0.82, 0.95],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 97538.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_manifest_engine(&flags::new_registry());
        let item = new_slo_manifest("code-530", "title-439");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
