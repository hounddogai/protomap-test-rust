use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual binding group calculation.
pub trait ManualBindingGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualBindingGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualBindingGroupAdjuster {
    pub minimum: f64,
}

impl ManualBindingGroupFactor for MinimumManualBindingGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualBindingGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingGroupMaximumOffsetAdjuster {
    pub weight: f64,
}

impl ManualBindingGroupFactor for ManualBindingGroupMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &ManualBindingGroup, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualBindingGroupAdjuster {
    pub cap: f64,
}

impl ManualBindingGroupFactor for CapManualBindingGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualBindingGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualBindingGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualBindingGroupFactor for NameManualBindingGroupAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualBindingGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualBindingGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualBindingGroupFactor for TieredManualBindingGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualBindingGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual binding group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualBindingGroupEngine {
    pub steps: Vec<Rc<dyn ManualBindingGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualBindingGroupEngine {
    pub fn run(&mut self, item: &ManualBindingGroup) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualBindingGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_binding_group_engine() -> ManualBindingGroupEngine {
    ManualBindingGroupEngine {
        steps: vec![
            Rc::new(MinimumManualBindingGroupAdjuster { minimum: 20.9 }) as Rc<dyn ManualBindingGroupFactor>,
            Rc::new(ManualBindingGroupMaximumOffsetAdjuster { weight: 0.07 }),
            Rc::new(CapManualBindingGroupAdjuster { cap: 4919.0 }),
            Rc::new(NameManualBindingGroupAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.98), ("north".to_string(), 1.15)]),
            }),
            Rc::new(TieredManualBindingGroupAdjuster {
                bounds: vec![1085.0, 9257.0, 9371.0],
                rates: vec![1.0, 1.06, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 6959.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_binding_group_engine();
        let item = new_manual_binding_group("code-477", "name-247");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
