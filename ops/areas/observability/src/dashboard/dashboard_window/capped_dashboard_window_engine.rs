use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped dashboard window calculation.
pub trait CappedDashboardWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedDashboardWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedDashboardWindowAdjuster {
    pub cap: f64,
}

impl CappedDashboardWindowModifier for CapCappedDashboardWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedDashboardWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedDashboardWindowAdjuster {
    pub step: f64,
}

impl CappedDashboardWindowModifier for RoundingCappedDashboardWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedDashboardWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedDashboardWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedDashboardWindowModifier for TieredCappedDashboardWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedDashboardWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardWindowEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl CappedDashboardWindowModifier for CappedDashboardWindowEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &CappedDashboardWindow, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceCappedDashboardWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedDashboardWindowModifier for NamespaceCappedDashboardWindowAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &CappedDashboardWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedDashboardWindowAdjuster {
    pub minimum: f64,
}

impl CappedDashboardWindowModifier for MinimumCappedDashboardWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedDashboardWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped dashboard window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedDashboardWindowEngine {
    pub steps: Vec<Rc<dyn CappedDashboardWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedDashboardWindowEngine {
    pub fn evaluate(&mut self, item: &CappedDashboardWindow) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedDashboardWindowModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedDashboardWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_dashboard_window_engine() -> CappedDashboardWindowEngine {
    CappedDashboardWindowEngine {
        steps: vec![
            Rc::new(CapCappedDashboardWindowAdjuster { cap: 17505.0 }) as Rc<dyn CappedDashboardWindowModifier>,
            Rc::new(RoundingCappedDashboardWindowAdjuster { step: 5.0 }),
            Rc::new(TieredCappedDashboardWindowAdjuster {
                bounds: vec![3441.0, 3726.0],
                rates: vec![0.97, 0.98],
            }),
            Rc::new(CappedDashboardWindowEndsAtOffsetAdjuster { weight: 0.2 }),
            Rc::new(NamespaceCappedDashboardWindowAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.36), ("north".to_string(), 1.24)]),
            }),
            Rc::new(MinimumCappedDashboardWindowAdjuster { minimum: 11.9 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 10071.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_dashboard_window_engine();
        let item = new_capped_dashboard_window("code-862", "title-503");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
