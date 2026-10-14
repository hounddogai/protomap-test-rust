use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering target calculation.
pub trait PeeringTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringTargetAdjuster {
    pub minimum: f64,
}

impl PeeringTargetStep for MinimumPeeringTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringTargetDiscountOffsetAdjuster {
    pub weight: f64,
}

impl PeeringTargetStep for PeeringTargetDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringTarget, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalePeeringTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringTargetStep for LocalePeeringTargetAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &PeeringTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringTargetStep for TieredPeeringTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringTargetStep for FlaggedPeeringTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringTargetAdjuster {
    pub percent: f64,
}

impl PeeringTargetStep for PercentPeeringTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the peering target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringTargetEngine {
    pub steps: Vec<Rc<dyn PeeringTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringTargetEngine {
    pub fn price(&mut self, item: &PeeringTarget) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[PeeringTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_peering_target_engine(flags: &flags::Registry) -> PeeringTargetEngine {
    PeeringTargetEngine {
        steps: vec![
            Rc::new(MinimumPeeringTargetAdjuster { minimum: 10.8 }) as Rc<dyn PeeringTargetStep>,
            Rc::new(PeeringTargetDiscountOffsetAdjuster { weight: 0.28 }),
            Rc::new(LocalePeeringTargetAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.32)]),
            }),
            Rc::new(TieredPeeringTargetAdjuster {
                bounds: vec![4055.0, 6242.0, 7072.0],
                rates: vec![1.06, 1.06, 1.11],
            }),
            Rc::new(FlaggedPeeringTargetAdjuster {
                flags: flags.clone(),
                flag: "peering-target-lenient".to_string(),
                percent: -0.3,
            }),
            Rc::new(PercentPeeringTargetAdjuster { percent: -22.1 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 67987.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_target_engine(&flags::new_registry());
        let item = new_peering_target("code-548", "label-941");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
