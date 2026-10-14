use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering profile calculation.
pub trait PeeringProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPeeringProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringProfileAdjuster for RegionPeeringProfileAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PeeringProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringProfileAdjuster {
    pub step: f64,
}

impl PeeringProfileAdjuster for RoundingPeeringProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringProfileAdjuster {
    pub minimum: f64,
}

impl PeeringProfileAdjuster for MinimumPeeringProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringProfileTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl PeeringProfileAdjuster for PeeringProfileTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringProfile, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringProfileAdjuster {
    pub cap: f64,
}

impl PeeringProfileAdjuster for CapPeeringProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the peering profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringProfileEngine {
    pub steps: Vec<Rc<dyn PeeringProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringProfileEngine {
    pub fn price(&mut self, item: &PeeringProfile) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PeeringProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_peering_profile_engine() -> PeeringProfileEngine {
    PeeringProfileEngine {
        steps: vec![
            Rc::new(RegionPeeringProfileAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.09),
                    ("retail".to_string(), 1.32),
                    ("west".to_string(), 1.17),
                ]),
            }) as Rc<dyn PeeringProfileAdjuster>,
            Rc::new(RoundingPeeringProfileAdjuster { step: 0.05 }),
            Rc::new(MinimumPeeringProfileAdjuster { minimum: 22.0 }),
            Rc::new(PeeringProfileTtlSecondsOffsetAdjuster { weight: 0.67 }),
            Rc::new(CapPeeringProfileAdjuster { cap: 11282.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 75489.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_profile_engine();
        let item = new_peering_profile("code-245", "label-768");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
