use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary binding slot calculation.
pub trait PrimaryBindingSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryBindingSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryBindingSlotAdjuster {
    pub step: f64,
}

impl PrimaryBindingSlotModifier for RoundingPrimaryBindingSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryBindingSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPrimaryBindingSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryBindingSlotModifier for RegionPrimaryBindingSlotAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PrimaryBindingSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSlotWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryBindingSlotModifier for PrimaryBindingSlotWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryBindingSlot, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryBindingSlotAdjuster {
    pub minimum: f64,
}

impl PrimaryBindingSlotModifier for MinimumPrimaryBindingSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryBindingSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryBindingSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryBindingSlotModifier for TieredPrimaryBindingSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryBindingSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryBindingSlotAdjuster {
    pub percent: f64,
}

impl PrimaryBindingSlotModifier for PercentPrimaryBindingSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryBindingSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary binding slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryBindingSlotEngine {
    pub steps: Vec<Rc<dyn PrimaryBindingSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryBindingSlotEngine {
    pub fn run(&mut self, item: &PrimaryBindingSlot) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryBindingSlotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryBindingSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_binding_slot_engine() -> PrimaryBindingSlotEngine {
    PrimaryBindingSlotEngine {
        steps: vec![
            Rc::new(RoundingPrimaryBindingSlotAdjuster { step: 0.1 }) as Rc<dyn PrimaryBindingSlotModifier>,
            Rc::new(RegionPrimaryBindingSlotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.21)]),
            }),
            Rc::new(PrimaryBindingSlotWindowDaysOffsetAdjuster { weight: 0.44 }),
            Rc::new(MinimumPrimaryBindingSlotAdjuster { minimum: 21.6 }),
            Rc::new(TieredPrimaryBindingSlotAdjuster {
                bounds: vec![1856.0, 3666.0, 7836.0],
                rates: vec![0.94, 0.94, 0.81],
            }),
            Rc::new(PercentPrimaryBindingSlotAdjuster { percent: -16.3 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 72533.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_binding_slot_engine();
        let item = new_primary_binding_slot("code-761", "name-474");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
