use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the manual region pin policy calculation.
pub trait ManualRegionPinPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualRegionPinPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualRegionPinPolicyAdjuster {
    pub cap: f64,
}

impl ManualRegionPinPolicyAdjuster for CapManualRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualRegionPinPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRegionPinPolicyWeightOffsetAdjuster {
    pub weight: f64,
}

impl ManualRegionPinPolicyAdjuster for ManualRegionPinPolicyWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &ManualRegionPinPolicy, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryManualRegionPinPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualRegionPinPolicyAdjuster for CategoryManualRegionPinPolicyAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ManualRegionPinPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the manual region pin policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualRegionPinPolicyEngine {
    pub steps: Vec<Rc<dyn ManualRegionPinPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualRegionPinPolicyEngine {
    pub fn run(&mut self, item: &ManualRegionPinPolicy) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualRegionPinPolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualRegionPinPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_region_pin_policy_engine() -> ManualRegionPinPolicyEngine {
    ManualRegionPinPolicyEngine {
        steps: vec![
            Rc::new(CapManualRegionPinPolicyAdjuster { cap: 38050.0 }) as Rc<dyn ManualRegionPinPolicyAdjuster>,
            Rc::new(ManualRegionPinPolicyWeightOffsetAdjuster { weight: 0.13 }),
            Rc::new(CategoryManualRegionPinPolicyAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.92),
                    ("central".to_string(), 1.07),
                    ("retail".to_string(), 0.92),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 48382.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_region_pin_policy_engine();
        let item = new_manual_region_pin_policy("code-143", "title-29");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
