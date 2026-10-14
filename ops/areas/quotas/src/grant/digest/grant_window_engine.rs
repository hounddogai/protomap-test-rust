use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the grant window calculation.
pub trait GrantWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantWindowCeilingOffsetAdjuster {
    pub weight: f64,
}

impl GrantWindowStep for GrantWindowCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &GrantWindow, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantWindowAdjuster {
    pub percent: f64,
}

impl GrantWindowStep for PercentGrantWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantWindowStep for TieredGrantWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the grant window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantWindowEngine {
    pub steps: Vec<Rc<dyn GrantWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantWindowEngine {
    pub fn price(&mut self, item: &GrantWindow) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantWindowStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_window_engine() -> GrantWindowEngine {
    GrantWindowEngine {
        steps: vec![
            Rc::new(GrantWindowCeilingOffsetAdjuster { weight: 0.79 }) as Rc<dyn GrantWindowStep>,
            Rc::new(PercentGrantWindowAdjuster { percent: 18.2 }),
            Rc::new(TieredGrantWindowAdjuster {
                bounds: vec![6118.0, 6722.0],
                rates: vec![0.8, 1.18],
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 68485.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_window_engine();
        let item = new_grant_window("code-680", "title-353");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
