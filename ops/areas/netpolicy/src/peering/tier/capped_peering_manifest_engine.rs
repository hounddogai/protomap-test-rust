use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped peering manifest calculation.
pub trait CappedPeeringManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedPeeringManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedPeeringManifestAdjuster {
    pub step: f64,
}

impl CappedPeeringManifestFactor for RoundingCappedPeeringManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedPeeringManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringManifestAmountOffsetAdjuster {
    pub weight: f64,
}

impl CappedPeeringManifestFactor for CappedPeeringManifestAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &CappedPeeringManifest, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedPeeringManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedPeeringManifestFactor for TieredCappedPeeringManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedPeeringManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCappedPeeringManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedPeeringManifestFactor for NameCappedPeeringManifestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CappedPeeringManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedPeeringManifestAdjuster {
    pub cap: f64,
}

impl CappedPeeringManifestFactor for CapCappedPeeringManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedPeeringManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped peering manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedPeeringManifestEngine {
    pub steps: Vec<Rc<dyn CappedPeeringManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedPeeringManifestEngine {
    pub fn run(&mut self, item: &CappedPeeringManifest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedPeeringManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_peering_manifest_engine() -> CappedPeeringManifestEngine {
    CappedPeeringManifestEngine {
        steps: vec![
            Rc::new(RoundingCappedPeeringManifestAdjuster { step: 1.0 }) as Rc<dyn CappedPeeringManifestFactor>,
            Rc::new(CappedPeeringManifestAmountOffsetAdjuster { weight: 0.4 }),
            Rc::new(TieredCappedPeeringManifestAdjuster {
                bounds: vec![1388.0, 4143.0, 7114.0, 7763.0],
                rates: vec![0.84, 1.08, 1.15, 0.9],
            }),
            Rc::new(NameCappedPeeringManifestAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.85),
                    ("west".to_string(), 0.89),
                    ("south".to_string(), 0.71),
                ]),
            }),
            Rc::new(CapCappedPeeringManifestAdjuster { cap: 30503.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 2652.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_peering_manifest_engine();
        let item = new_capped_peering_manifest("code-32", "name-906");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
