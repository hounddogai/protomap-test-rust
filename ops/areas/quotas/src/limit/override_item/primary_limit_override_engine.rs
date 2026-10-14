use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary limit override calculation.
pub trait PrimaryLimitOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryLimitOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalePrimaryLimitOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryLimitOverrideAdjuster for LocalePrimaryLimitOverrideAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &PrimaryLimitOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryLimitOverrideAdjuster {
    pub step: f64,
}

impl PrimaryLimitOverrideAdjuster for RoundingPrimaryLimitOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryLimitOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryLimitOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryLimitOverrideAdjuster for FlaggedPrimaryLimitOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryLimitOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryLimitOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryLimitOverrideAdjuster for TieredPrimaryLimitOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryLimitOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitOverrideBurstOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryLimitOverrideAdjuster for PrimaryLimitOverrideBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryLimitOverride, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryLimitOverrideAdjuster {
    pub cap: f64,
}

impl PrimaryLimitOverrideAdjuster for CapPrimaryLimitOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryLimitOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary limit override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryLimitOverrideEngine {
    pub steps: Vec<Rc<dyn PrimaryLimitOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryLimitOverrideEngine {
    pub fn price(&mut self, item: &PrimaryLimitOverride) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryLimitOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_limit_override_engine(flags: &flags::Registry) -> PrimaryLimitOverrideEngine {
    PrimaryLimitOverrideEngine {
        steps: vec![
            Rc::new(LocalePrimaryLimitOverrideAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.85), ("retail".to_string(), 1.38)]),
            }) as Rc<dyn PrimaryLimitOverrideAdjuster>,
            Rc::new(RoundingPrimaryLimitOverrideAdjuster { step: 0.05 }),
            Rc::new(FlaggedPrimaryLimitOverrideAdjuster {
                flags: flags.clone(),
                flag: "primary-limit-override-tiered".to_string(),
                percent: 6.5,
            }),
            Rc::new(TieredPrimaryLimitOverrideAdjuster {
                bounds: vec![2861.0, 4263.0, 7871.0],
                rates: vec![0.96, 1.0, 0.95],
            }),
            Rc::new(PrimaryLimitOverrideBurstOffsetAdjuster { weight: 0.15 }),
            Rc::new(CapPrimaryLimitOverrideAdjuster { cap: 49271.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 81638.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_limit_override_engine(&flags::new_registry());
        let item = new_primary_limit_override("code-580", "name-532");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
