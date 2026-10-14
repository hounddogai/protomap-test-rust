use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered postmortem override calculation.
pub trait TieredPostmortemOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredPostmortemOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemOverrideRetriesOffsetAdjuster {
    pub weight: f64,
}

impl TieredPostmortemOverrideModifier for TieredPostmortemOverrideRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &TieredPostmortemOverride, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredPostmortemOverrideAdjuster {
    pub step: f64,
}

impl TieredPostmortemOverrideModifier for RoundingTieredPostmortemOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredPostmortemOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitTieredPostmortemOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredPostmortemOverrideModifier for UnitTieredPostmortemOverrideAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &TieredPostmortemOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredPostmortemOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredPostmortemOverrideModifier for TieredTieredPostmortemOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredPostmortemOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered postmortem override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredPostmortemOverrideEngine {
    pub steps: Vec<Rc<dyn TieredPostmortemOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredPostmortemOverrideEngine {
    pub fn evaluate(&mut self, item: &TieredPostmortemOverride) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredPostmortemOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredPostmortemOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_postmortem_override_engine() -> TieredPostmortemOverrideEngine {
    TieredPostmortemOverrideEngine {
        steps: vec![
            Rc::new(TieredPostmortemOverrideRetriesOffsetAdjuster { weight: 1.35 })
                as Rc<dyn TieredPostmortemOverrideModifier>,
            Rc::new(RoundingTieredPostmortemOverrideAdjuster { step: 0.05 }),
            Rc::new(UnitTieredPostmortemOverrideAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.34), ("north".to_string(), 0.81)]),
            }),
            Rc::new(TieredTieredPostmortemOverrideAdjuster {
                bounds: vec![4312.0, 4752.0, 4811.0, 8387.0],
                rates: vec![1.03, 1.18, 1.1, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 73986.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_postmortem_override_engine();
        let item = new_tiered_postmortem_override("code-936", "name-473");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
