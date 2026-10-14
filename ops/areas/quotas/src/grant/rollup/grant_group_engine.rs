use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant group calculation.
pub trait GrantGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelGrantGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantGroupAdjuster for ChannelGrantGroupAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &GrantGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantGroupAdjuster {
    pub cap: f64,
}

impl GrantGroupAdjuster for CapGrantGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantGroupAdjuster for TieredGrantGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantGroupGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl GrantGroupAdjuster for GrantGroupGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &GrantGroup, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the grant group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantGroupEngine {
    pub steps: Vec<Rc<dyn GrantGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantGroupEngine {
    pub fn run(&mut self, item: &GrantGroup) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[GrantGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_grant_group_engine() -> GrantGroupEngine {
    GrantGroupEngine {
        steps: vec![
            Rc::new(ChannelGrantGroupAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.81),
                    ("retail".to_string(), 1.0),
                    ("north".to_string(), 0.74),
                ]),
            }) as Rc<dyn GrantGroupAdjuster>,
            Rc::new(CapGrantGroupAdjuster { cap: 37454.0 }),
            Rc::new(TieredGrantGroupAdjuster {
                bounds: vec![3697.0, 4430.0, 9775.0],
                rates: vec![1.1, 1.14, 1.09],
            }),
            Rc::new(GrantGroupGraceDaysOffsetAdjuster { weight: 0.21 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 18268.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_group_engine();
        let item = new_grant_group("code-983", "name-481");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
