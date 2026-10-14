use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted federation group calculation.
pub trait WeightedFederationGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFederationGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedFederationGroupAdjuster {
    pub step: f64,
}

impl WeightedFederationGroupModifier for RoundingWeightedFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedFederationGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedFederationGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedFederationGroupModifier for TieredWeightedFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedFederationGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedFederationGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedFederationGroupModifier for FlaggedWeightedFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedFederationGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedFederationGroupAdjuster {
    pub cap: f64,
}

impl WeightedFederationGroupModifier for CapWeightedFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedFederationGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedFederationGroupAdjuster {
    pub minimum: f64,
}

impl WeightedFederationGroupModifier for MinimumWeightedFederationGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedFederationGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationGroupWeightOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFederationGroupModifier for WeightedFederationGroupWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFederationGroup, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the weighted federation group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFederationGroupEngine {
    pub steps: Vec<Rc<dyn WeightedFederationGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFederationGroupEngine {
    pub fn evaluate(&mut self, item: &WeightedFederationGroup) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedFederationGroupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFederationGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_federation_group_engine(flags: &flags::Registry) -> WeightedFederationGroupEngine {
    WeightedFederationGroupEngine {
        steps: vec![
            Rc::new(RoundingWeightedFederationGroupAdjuster { step: 5.0 }) as Rc<dyn WeightedFederationGroupModifier>,
            Rc::new(TieredWeightedFederationGroupAdjuster {
                bounds: vec![959.0, 2413.0, 6557.0, 9438.0],
                rates: vec![0.85, 1.12, 0.99, 1.15],
            }),
            Rc::new(FlaggedWeightedFederationGroupAdjuster {
                flags: flags.clone(),
                flag: "weighted-federation-group-weighted".to_string(),
                percent: -8.5,
            }),
            Rc::new(CapWeightedFederationGroupAdjuster { cap: 7361.0 }),
            Rc::new(MinimumWeightedFederationGroupAdjuster { minimum: 4.1 }),
            Rc::new(WeightedFederationGroupWeightOffsetAdjuster { weight: 1.52 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 5480.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_federation_group_engine(&flags::new_registry());
        let item = new_weighted_federation_group("code-167", "name-947");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
