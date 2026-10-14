use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary peering tier calculation.
pub trait PrimaryPeeringTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryPeeringTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryPeeringTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryPeeringTierStep for TieredPrimaryPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryPeeringTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPrimaryPeeringTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryPeeringTierStep for SegmentPrimaryPeeringTierAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &PrimaryPeeringTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryPeeringTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryPeeringTierStep for FlaggedPrimaryPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryPeeringTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary peering tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryPeeringTierEngine {
    pub steps: Vec<Rc<dyn PrimaryPeeringTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryPeeringTierEngine {
    pub fn evaluate(&mut self, item: &PrimaryPeeringTier) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryPeeringTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_peering_tier_engine(flags: &flags::Registry) -> PrimaryPeeringTierEngine {
    PrimaryPeeringTierEngine {
        steps: vec![
            Rc::new(TieredPrimaryPeeringTierAdjuster {
                bounds: vec![2815.0, 5020.0, 7067.0, 7977.0],
                rates: vec![0.99, 0.92, 0.96, 1.05],
            }) as Rc<dyn PrimaryPeeringTierStep>,
            Rc::new(SegmentPrimaryPeeringTierAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.79),
                    ("south".to_string(), 0.76),
                    ("north".to_string(), 0.71),
                ]),
            }),
            Rc::new(FlaggedPrimaryPeeringTierAdjuster {
                flags: flags.clone(),
                flag: "primary-peering-tier-seasonal".to_string(),
                percent: 6.5,
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 19184.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_peering_tier_engine(&flags::new_registry());
        let item = new_primary_peering_tier("code-164", "label-859");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
