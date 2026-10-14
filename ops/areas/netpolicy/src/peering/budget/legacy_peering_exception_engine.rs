use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy peering exception calculation.
pub trait LegacyPeeringExceptionFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyPeeringException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringExceptionPriorityOffsetAdjuster {
    pub weight: f64,
}

impl LegacyPeeringExceptionFactor for LegacyPeeringExceptionPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyPeeringException, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLegacyPeeringExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyPeeringExceptionFactor for LabelLegacyPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LegacyPeeringException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyPeeringExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyPeeringExceptionFactor for TieredLegacyPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyPeeringException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyPeeringExceptionAdjuster {
    pub percent: f64,
}

impl LegacyPeeringExceptionFactor for PercentLegacyPeeringExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyPeeringException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy peering exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyPeeringExceptionEngine {
    pub steps: Vec<Rc<dyn LegacyPeeringExceptionFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyPeeringExceptionEngine {
    pub fn price(&mut self, item: &LegacyPeeringException) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyPeeringExceptionFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyPeeringException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_peering_exception_engine() -> LegacyPeeringExceptionEngine {
    LegacyPeeringExceptionEngine {
        steps: vec![
            Rc::new(LegacyPeeringExceptionPriorityOffsetAdjuster { weight: 1.43 })
                as Rc<dyn LegacyPeeringExceptionFactor>,
            Rc::new(LabelLegacyPeeringExceptionAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.25), ("online".to_string(), 1.09)]),
            }),
            Rc::new(TieredLegacyPeeringExceptionAdjuster {
                bounds: vec![5165.0, 8794.0, 9304.0, 9494.0],
                rates: vec![1.09, 1.02, 1.1, 0.97],
            }),
            Rc::new(PercentLegacyPeeringExceptionAdjuster { percent: -13.5 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 73318.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_peering_exception_engine();
        let item = new_legacy_peering_exception("code-558", "label-196");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
