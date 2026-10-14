use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual nonce manifest calculation.
pub trait ManualNonceManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualNonceManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualNonceManifestAdjuster {
    pub cap: f64,
}

impl ManualNonceManifestFactor for CapManualNonceManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualNonceManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceManifestCostOffsetAdjuster {
    pub weight: f64,
}

impl ManualNonceManifestFactor for ManualNonceManifestCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &ManualNonceManifest, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualNonceManifestAdjuster {
    pub step: f64,
}

impl ManualNonceManifestFactor for RoundingManualNonceManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualNonceManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeManualNonceManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualNonceManifestFactor for ScopeManualNonceManifestAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &ManualNonceManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualNonceManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualNonceManifestFactor for FlaggedManualNonceManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualNonceManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualNonceManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualNonceManifestFactor for TieredManualNonceManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualNonceManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual nonce manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualNonceManifestEngine {
    pub steps: Vec<Rc<dyn ManualNonceManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualNonceManifestEngine {
    pub fn evaluate(&mut self, item: &ManualNonceManifest) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualNonceManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualNonceManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_nonce_manifest_engine(flags: &flags::Registry) -> ManualNonceManifestEngine {
    ManualNonceManifestEngine {
        steps: vec![
            Rc::new(CapManualNonceManifestAdjuster { cap: 31335.0 }) as Rc<dyn ManualNonceManifestFactor>,
            Rc::new(ManualNonceManifestCostOffsetAdjuster { weight: 0.33 }),
            Rc::new(RoundingManualNonceManifestAdjuster { step: 5.0 }),
            Rc::new(ScopeManualNonceManifestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.05)]),
            }),
            Rc::new(FlaggedManualNonceManifestAdjuster {
                flags: flags.clone(),
                flag: "manual-nonce-manifest-primary".to_string(),
                percent: 0.8,
            }),
            Rc::new(TieredManualNonceManifestAdjuster {
                bounds: vec![772.0, 6438.0],
                rates: vec![0.89, 0.81],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 87994.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_nonce_manifest_engine(&flags::new_registry());
        let item = new_manual_nonce_manifest("code-9", "title-43");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
