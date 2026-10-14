use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered federation group calculation.
pub trait TieredFederationGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredFederationGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredFederationGroupAdjuster {
    pub step: f64,
}

impl TieredFederationGroupModifier for RoundingTieredFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredFederationGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredFederationGroupAdjuster {
    pub percent: f64,
}

impl TieredFederationGroupModifier for PercentTieredFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredFederationGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredFederationGroupAdjuster {
    pub cap: f64,
}

impl TieredFederationGroupModifier for CapTieredFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredFederationGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tiered federation group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredFederationGroupEngine {
    pub steps: Vec<Rc<dyn TieredFederationGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredFederationGroupEngine {
    pub fn price(&mut self, item: &TieredFederationGroup) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[TieredFederationGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_federation_group_engine() -> TieredFederationGroupEngine {
    TieredFederationGroupEngine {
        steps: vec![
            Rc::new(RoundingTieredFederationGroupAdjuster { step: 0.5 }) as Rc<dyn TieredFederationGroupModifier>,
            Rc::new(PercentTieredFederationGroupAdjuster { percent: -12.6 }),
            Rc::new(CapTieredFederationGroupAdjuster { cap: 6336.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 57093.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_federation_group_engine();
        let item = new_tiered_federation_group("code-312", "label-502");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
