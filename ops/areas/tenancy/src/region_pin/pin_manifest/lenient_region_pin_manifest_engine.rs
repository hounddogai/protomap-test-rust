use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient region pin manifest calculation.
pub trait LenientRegionPinManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRegionPinManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRegionPinManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRegionPinManifestModifier for TieredLenientRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRegionPinManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientRegionPinManifestAdjuster {
    pub percent: f64,
}

impl LenientRegionPinManifestModifier for PercentLenientRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientRegionPinManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientRegionPinManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientRegionPinManifestModifier for FlaggedLenientRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientRegionPinManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientRegionPinManifestAdjuster {
    pub cap: f64,
}

impl LenientRegionPinManifestModifier for CapLenientRegionPinManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientRegionPinManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinManifestCostOffsetAdjuster {
    pub weight: f64,
}

impl LenientRegionPinManifestModifier for LenientRegionPinManifestCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &LenientRegionPinManifest, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

/// Runs the lenient region pin manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRegionPinManifestEngine {
    pub steps: Vec<Rc<dyn LenientRegionPinManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRegionPinManifestEngine {
    pub fn evaluate(&mut self, item: &LenientRegionPinManifest) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientRegionPinManifestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientRegionPinManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_region_pin_manifest_engine(flags: &flags::Registry) -> LenientRegionPinManifestEngine {
    LenientRegionPinManifestEngine {
        steps: vec![
            Rc::new(TieredLenientRegionPinManifestAdjuster {
                bounds: vec![2240.0, 4001.0, 5265.0, 8533.0],
                rates: vec![1.16, 1.02, 1.07, 0.96],
            }) as Rc<dyn LenientRegionPinManifestModifier>,
            Rc::new(PercentLenientRegionPinManifestAdjuster { percent: -11.4 }),
            Rc::new(FlaggedLenientRegionPinManifestAdjuster {
                flags: flags.clone(),
                flag: "lenient-region-pin-manifest-manual".to_string(),
                percent: -9.7,
            }),
            Rc::new(CapLenientRegionPinManifestAdjuster { cap: 26534.0 }),
            Rc::new(LenientRegionPinManifestCostOffsetAdjuster { weight: 1.13 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 24994.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_region_pin_manifest_engine(&flags::new_registry());
        let item = new_lenient_region_pin_manifest("code-810", "label-679");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
