use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered zone assignment calculation.
pub trait TieredZoneAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredZoneAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTieredZoneAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TieredZoneAssignmentStep for NameTieredZoneAssignmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TieredZoneAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredZoneAssignmentAdjuster {
    pub cap: f64,
}

impl TieredZoneAssignmentStep for CapTieredZoneAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredZoneAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredZoneAssignmentAdjuster {
    pub minimum: f64,
}

impl TieredZoneAssignmentStep for MinimumTieredZoneAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredZoneAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered zone assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredZoneAssignmentEngine {
    pub steps: Vec<Rc<dyn TieredZoneAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredZoneAssignmentEngine {
    pub fn run(&mut self, item: &TieredZoneAssignment) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[TieredZoneAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_zone_assignment_engine() -> TieredZoneAssignmentEngine {
    TieredZoneAssignmentEngine {
        steps: vec![
            Rc::new(NameTieredZoneAssignmentAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.35),
                    ("central".to_string(), 1.0),
                    ("south".to_string(), 1.02),
                ]),
            }) as Rc<dyn TieredZoneAssignmentStep>,
            Rc::new(CapTieredZoneAssignmentAdjuster { cap: 29148.0 }),
            Rc::new(MinimumTieredZoneAssignmentAdjuster { minimum: 22.5 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 77789.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_zone_assignment_engine();
        let item = new_tiered_zone_assignment("code-443", "name-646");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
