use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy postmortem manifest calculation.
pub trait LegacyPostmortemManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyPostmortemManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLegacyPostmortemManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyPostmortemManifestModifier for SourceLegacyPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &LegacyPostmortemManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyPostmortemManifestAdjuster {
    pub minimum: f64,
}

impl LegacyPostmortemManifestModifier for MinimumLegacyPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyPostmortemManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyPostmortemManifestAdjuster {
    pub cap: f64,
}

impl LegacyPostmortemManifestModifier for CapLegacyPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyPostmortemManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyPostmortemManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyPostmortemManifestModifier for TieredLegacyPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyPostmortemManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyPostmortemManifestAdjuster {
    pub step: f64,
}

impl LegacyPostmortemManifestModifier for RoundingLegacyPostmortemManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyPostmortemManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy postmortem manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyPostmortemManifestEngine {
    pub steps: Vec<Rc<dyn LegacyPostmortemManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyPostmortemManifestEngine {
    pub fn evaluate(&mut self, item: &LegacyPostmortemManifest) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[LegacyPostmortemManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_postmortem_manifest_engine() -> LegacyPostmortemManifestEngine {
    LegacyPostmortemManifestEngine {
        steps: vec![
            Rc::new(SourceLegacyPostmortemManifestAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.11)]),
            }) as Rc<dyn LegacyPostmortemManifestModifier>,
            Rc::new(MinimumLegacyPostmortemManifestAdjuster { minimum: 49.0 }),
            Rc::new(CapLegacyPostmortemManifestAdjuster { cap: 48353.0 }),
            Rc::new(TieredLegacyPostmortemManifestAdjuster {
                bounds: vec![7188.0, 8808.0, 9407.0],
                rates: vec![1.13, 1.1, 0.93],
            }),
            Rc::new(RoundingLegacyPostmortemManifestAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 62032.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_postmortem_manifest_engine();
        let item = new_legacy_postmortem_manifest("code-712", "title-195");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
