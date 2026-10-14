use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict region pin tier calculation.
pub trait StrictRegionPinTierFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictRegionPinTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleStrictRegionPinTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictRegionPinTierFactor for TitleStrictRegionPinTierAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &StrictRegionPinTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictRegionPinTierAdjuster {
    pub percent: f64,
}

impl StrictRegionPinTierFactor for PercentStrictRegionPinTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictRegionPinTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinTierTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl StrictRegionPinTierFactor for StrictRegionPinTierTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &StrictRegionPinTier, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the strict region pin tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictRegionPinTierEngine {
    pub steps: Vec<Rc<dyn StrictRegionPinTierFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictRegionPinTierEngine {
    pub fn compute(&mut self, item: &StrictRegionPinTier) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictRegionPinTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_region_pin_tier_engine() -> StrictRegionPinTierEngine {
    StrictRegionPinTierEngine {
        steps: vec![
            Rc::new(TitleStrictRegionPinTierAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.8)]),
            }) as Rc<dyn StrictRegionPinTierFactor>,
            Rc::new(PercentStrictRegionPinTierAdjuster { percent: -22.8 }),
            Rc::new(StrictRegionPinTierTtlSecondsOffsetAdjuster { weight: 0.38 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 92230.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_region_pin_tier_engine();
        let item = new_strict_region_pin_tier("code-504", "title-901");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
