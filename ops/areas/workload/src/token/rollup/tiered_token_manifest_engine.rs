use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tiered token manifest calculation.
pub trait TieredTokenManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredTokenManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredTokenManifestAdjuster {
    pub minimum: f64,
}

impl TieredTokenManifestStep for MinimumTieredTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredTokenManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredTokenManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredTokenManifestStep for TieredTieredTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredTokenManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredTokenManifestAdjuster {
    pub percent: f64,
}

impl TieredTokenManifestStep for PercentTieredTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredTokenManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenManifestTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl TieredTokenManifestStep for TieredTokenManifestTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &TieredTokenManifest, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredTokenManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredTokenManifestStep for FlaggedTieredTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredTokenManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered token manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredTokenManifestEngine {
    pub steps: Vec<Rc<dyn TieredTokenManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredTokenManifestEngine {
    pub fn evaluate(&mut self, item: &TieredTokenManifest) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TieredTokenManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_token_manifest_engine(flags: &flags::Registry) -> TieredTokenManifestEngine {
    TieredTokenManifestEngine {
        steps: vec![
            Rc::new(MinimumTieredTokenManifestAdjuster { minimum: 8.6 }) as Rc<dyn TieredTokenManifestStep>,
            Rc::new(TieredTieredTokenManifestAdjuster {
                bounds: vec![4408.0, 7291.0, 8220.0],
                rates: vec![1.02, 1.07, 0.99],
            }),
            Rc::new(PercentTieredTokenManifestAdjuster { percent: -21.1 }),
            Rc::new(TieredTokenManifestTtlSecondsOffsetAdjuster { weight: 1.8 }),
            Rc::new(FlaggedTieredTokenManifestAdjuster {
                flags: flags.clone(),
                flag: "tiered-token-manifest-strict".to_string(),
                percent: 6.0,
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 85727.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_token_manifest_engine(&flags::new_registry());
        let item = new_tiered_token_manifest("code-275", "title-532");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
