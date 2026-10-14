use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary overage window calculation.
pub trait PrimaryOverageWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryOverageWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryOverageWindowAdjuster {
    pub minimum: f64,
}

impl PrimaryOverageWindowModifier for MinimumPrimaryOverageWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryOverageWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryOverageWindowAdjuster {
    pub percent: f64,
}

impl PrimaryOverageWindowModifier for PercentPrimaryOverageWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryOverageWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryOverageWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryOverageWindowModifier for TieredPrimaryOverageWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryOverageWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryOverageWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryOverageWindowModifier for SourcePrimaryOverageWindowAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryOverageWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary overage window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryOverageWindowEngine {
    pub steps: Vec<Rc<dyn PrimaryOverageWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryOverageWindowEngine {
    pub fn run(&mut self, item: &PrimaryOverageWindow) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryOverageWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_overage_window_engine() -> PrimaryOverageWindowEngine {
    PrimaryOverageWindowEngine {
        steps: vec![
            Rc::new(MinimumPrimaryOverageWindowAdjuster { minimum: 25.7 }) as Rc<dyn PrimaryOverageWindowModifier>,
            Rc::new(PercentPrimaryOverageWindowAdjuster { percent: -20.5 }),
            Rc::new(TieredPrimaryOverageWindowAdjuster {
                bounds: vec![7573.0, 8429.0],
                rates: vec![0.91, 1.01],
            }),
            Rc::new(SourcePrimaryOverageWindowAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.12), ("north".to_string(), 1.27)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 91830.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_overage_window_engine();
        let item = new_primary_overage_window("code-72", "title-655");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
