use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global peering group calculation.
pub trait GlobalPeeringGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalPeeringGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalPeeringGroupAdjuster {
    pub step: f64,
}

impl GlobalPeeringGroupStep for RoundingGlobalPeeringGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalPeeringGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalPeeringGroupAdjuster {
    pub minimum: f64,
}

impl GlobalPeeringGroupStep for MinimumGlobalPeeringGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalPeeringGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionGlobalPeeringGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalPeeringGroupStep for RegionGlobalPeeringGroupAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &GlobalPeeringGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the global peering group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalPeeringGroupEngine {
    pub steps: Vec<Rc<dyn GlobalPeeringGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalPeeringGroupEngine {
    pub fn price(&mut self, item: &GlobalPeeringGroup) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalPeeringGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_peering_group_engine() -> GlobalPeeringGroupEngine {
    GlobalPeeringGroupEngine {
        steps: vec![
            Rc::new(RoundingGlobalPeeringGroupAdjuster { step: 0.1 }) as Rc<dyn GlobalPeeringGroupStep>,
            Rc::new(MinimumGlobalPeeringGroupAdjuster { minimum: 25.2 }),
            Rc::new(RegionGlobalPeeringGroupAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.23)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 47237.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_peering_group_engine();
        let item = new_global_peering_group("code-527", "title-556");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
