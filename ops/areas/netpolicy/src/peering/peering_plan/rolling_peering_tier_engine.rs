use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling peering tier calculation.
pub trait RollingPeeringTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPeeringTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingPeeringTierAdjuster {
    pub step: f64,
}

impl RollingPeeringTierStep for RoundingRollingPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingPeeringTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRollingPeeringTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingPeeringTierStep for LocaleRollingPeeringTierAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RollingPeeringTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingPeeringTierAdjuster {
    pub cap: f64,
}

impl RollingPeeringTierStep for CapRollingPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingPeeringTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingPeeringTierAdjuster {
    pub minimum: f64,
}

impl RollingPeeringTierStep for MinimumRollingPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingPeeringTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingPeeringTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingPeeringTierStep for FlaggedRollingPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingPeeringTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPeeringTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPeeringTierStep for TieredRollingPeeringTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPeeringTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling peering tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPeeringTierEngine {
    pub steps: Vec<Rc<dyn RollingPeeringTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPeeringTierEngine {
    pub fn compute(&mut self, item: &RollingPeeringTier) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[RollingPeeringTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_peering_tier_engine(flags: &flags::Registry) -> RollingPeeringTierEngine {
    RollingPeeringTierEngine {
        steps: vec![
            Rc::new(RoundingRollingPeeringTierAdjuster { step: 0.5 }) as Rc<dyn RollingPeeringTierStep>,
            Rc::new(LocaleRollingPeeringTierAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.94),
                    ("east".to_string(), 1.13),
                    ("south".to_string(), 0.87),
                ]),
            }),
            Rc::new(CapRollingPeeringTierAdjuster { cap: 10317.0 }),
            Rc::new(MinimumRollingPeeringTierAdjuster { minimum: 17.7 }),
            Rc::new(FlaggedRollingPeeringTierAdjuster {
                flags: flags.clone(),
                flag: "rolling-peering-tier-pending".to_string(),
                percent: -6.4,
            }),
            Rc::new(TieredRollingPeeringTierAdjuster {
                bounds: vec![4198.0, 6395.0],
                rates: vec![1.04, 1.09],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 2052.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_peering_tier_engine(&flags::new_registry());
        let item = new_rolling_peering_tier("code-358", "name-782");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
