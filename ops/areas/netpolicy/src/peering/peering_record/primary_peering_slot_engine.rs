use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary peering slot calculation.
pub trait PrimaryPeeringSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryPeeringSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryPeeringSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryPeeringSlotAdjuster for FlaggedPrimaryPeeringSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryPeeringSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringSlotThresholdOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryPeeringSlotAdjuster for PrimaryPeeringSlotThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryPeeringSlot, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryPeeringSlotAdjuster {
    pub percent: f64,
}

impl PrimaryPeeringSlotAdjuster for PercentPrimaryPeeringSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryPeeringSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryPeeringSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryPeeringSlotAdjuster for TieredPrimaryPeeringSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryPeeringSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryPeeringSlotAdjuster {
    pub minimum: f64,
}

impl PrimaryPeeringSlotAdjuster for MinimumPrimaryPeeringSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryPeeringSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary peering slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryPeeringSlotEngine {
    pub steps: Vec<Rc<dyn PrimaryPeeringSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryPeeringSlotEngine {
    pub fn compute(&mut self, item: &PrimaryPeeringSlot) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryPeeringSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_peering_slot_engine(flags: &flags::Registry) -> PrimaryPeeringSlotEngine {
    PrimaryPeeringSlotEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryPeeringSlotAdjuster {
                flags: flags.clone(),
                flag: "primary-peering-slot-default".to_string(),
                percent: 2.3,
            }) as Rc<dyn PrimaryPeeringSlotAdjuster>,
            Rc::new(PrimaryPeeringSlotThresholdOffsetAdjuster { weight: 0.56 }),
            Rc::new(PercentPrimaryPeeringSlotAdjuster { percent: -22.2 }),
            Rc::new(TieredPrimaryPeeringSlotAdjuster {
                bounds: vec![3620.0, 7554.0],
                rates: vec![1.18, 1.02],
            }),
            Rc::new(MinimumPrimaryPeeringSlotAdjuster { minimum: 48.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 42173.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_peering_slot_engine(&flags::new_registry());
        let item = new_primary_peering_slot("code-662", "title-780");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
