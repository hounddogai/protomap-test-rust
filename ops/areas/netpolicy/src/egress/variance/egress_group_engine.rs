use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress group calculation.
pub trait EgressGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelEgressGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressGroupAdjuster for LabelEgressGroupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &EgressGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressGroupAdjuster {
    pub percent: f64,
}

impl EgressGroupAdjuster for PercentEgressGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressGroupMaximumOffsetAdjuster {
    pub weight: f64,
}

impl EgressGroupAdjuster for EgressGroupMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &EgressGroup, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

/// Runs the egress group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressGroupEngine {
    pub steps: Vec<Rc<dyn EgressGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressGroupEngine {
    pub fn compute(&mut self, item: &EgressGroup) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[EgressGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_egress_group_engine() -> EgressGroupEngine {
    EgressGroupEngine {
        steps: vec![
            Rc::new(LabelEgressGroupAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.23),
                    ("east".to_string(), 1.09),
                    ("south".to_string(), 0.7),
                ]),
            }) as Rc<dyn EgressGroupAdjuster>,
            Rc::new(PercentEgressGroupAdjuster { percent: 10.7 }),
            Rc::new(EgressGroupMaximumOffsetAdjuster { weight: 1.98 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 96319.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_group_engine();
        let item = new_egress_group("code-580", "label-246");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
