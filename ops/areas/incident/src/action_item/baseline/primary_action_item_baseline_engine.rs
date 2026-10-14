use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary action item baseline calculation.
pub trait PrimaryActionItemBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryActionItemBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryActionItemBaselineAdjuster {
    pub minimum: f64,
}

impl PrimaryActionItemBaselineAdjuster for MinimumPrimaryActionItemBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryActionItemBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBaselineGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryActionItemBaselineAdjuster for PrimaryActionItemBaselineGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryActionItemBaseline, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryActionItemBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryActionItemBaselineAdjuster for TieredPrimaryActionItemBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryActionItemBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryActionItemBaselineAdjuster {
    pub percent: f64,
}

impl PrimaryActionItemBaselineAdjuster for PercentPrimaryActionItemBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryActionItemBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryPrimaryActionItemBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryActionItemBaselineAdjuster for CountryPrimaryActionItemBaselineAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &PrimaryActionItemBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryActionItemBaselineAdjuster {
    pub cap: f64,
}

impl PrimaryActionItemBaselineAdjuster for CapPrimaryActionItemBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryActionItemBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary action item baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryActionItemBaselineEngine {
    pub steps: Vec<Rc<dyn PrimaryActionItemBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryActionItemBaselineEngine {
    pub fn compute(&mut self, item: &PrimaryActionItemBaseline) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryActionItemBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_action_item_baseline_engine() -> PrimaryActionItemBaselineEngine {
    PrimaryActionItemBaselineEngine {
        steps: vec![
            Rc::new(MinimumPrimaryActionItemBaselineAdjuster { minimum: 17.2 })
                as Rc<dyn PrimaryActionItemBaselineAdjuster>,
            Rc::new(PrimaryActionItemBaselineGraceDaysOffsetAdjuster { weight: 1.09 }),
            Rc::new(TieredPrimaryActionItemBaselineAdjuster {
                bounds: vec![1017.0, 1870.0, 3488.0, 8742.0],
                rates: vec![1.2, 0.98, 0.97, 1.11],
            }),
            Rc::new(PercentPrimaryActionItemBaselineAdjuster { percent: -17.0 }),
            Rc::new(CountryPrimaryActionItemBaselineAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.37), ("online".to_string(), 0.86)]),
            }),
            Rc::new(CapPrimaryActionItemBaselineAdjuster { cap: 40122.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 63763.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_action_item_baseline_engine();
        let item = new_primary_action_item_baseline("code-933", "title-185");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
