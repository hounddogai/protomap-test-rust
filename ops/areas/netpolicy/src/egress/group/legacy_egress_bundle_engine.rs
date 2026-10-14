use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy egress bundle calculation.
pub trait LegacyEgressBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyEgressBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyEgressBundleAdjuster {
    pub minimum: f64,
}

impl LegacyEgressBundleModifier for MinimumLegacyEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyEgressBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEgressBundleScoreOffsetAdjuster {
    pub weight: f64,
}

impl LegacyEgressBundleModifier for LegacyEgressBundleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyEgressBundle, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyEgressBundleAdjuster {
    pub cap: f64,
}

impl LegacyEgressBundleModifier for CapLegacyEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyEgressBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLegacyEgressBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyEgressBundleModifier for UnitLegacyEgressBundleAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LegacyEgressBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyEgressBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyEgressBundleModifier for TieredLegacyEgressBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyEgressBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the legacy egress bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyEgressBundleEngine {
    pub steps: Vec<Rc<dyn LegacyEgressBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyEgressBundleEngine {
    pub fn evaluate(&mut self, item: &LegacyEgressBundle) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[LegacyEgressBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_egress_bundle_engine() -> LegacyEgressBundleEngine {
    LegacyEgressBundleEngine {
        steps: vec![
            Rc::new(MinimumLegacyEgressBundleAdjuster { minimum: 17.2 }) as Rc<dyn LegacyEgressBundleModifier>,
            Rc::new(LegacyEgressBundleScoreOffsetAdjuster { weight: 0.18 }),
            Rc::new(CapLegacyEgressBundleAdjuster { cap: 31554.0 }),
            Rc::new(UnitLegacyEgressBundleAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.91), ("west".to_string(), 1.3)]),
            }),
            Rc::new(TieredLegacyEgressBundleAdjuster {
                bounds: vec![341.0, 3889.0, 4623.0],
                rates: vec![1.16, 0.89, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 92221.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_egress_bundle_engine();
        let item = new_legacy_egress_bundle("code-84", "title-485");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
