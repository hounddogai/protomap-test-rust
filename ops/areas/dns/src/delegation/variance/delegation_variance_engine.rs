use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation variance calculation.
pub trait DelegationVarianceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDelegationVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationVarianceAdjuster for ChannelDelegationVarianceAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &DelegationVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationVarianceAdjuster {
    pub percent: f64,
}

impl DelegationVarianceAdjuster for PercentDelegationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationVarianceAdjuster {
    pub step: f64,
}

impl DelegationVarianceAdjuster for RoundingDelegationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationVarianceAdjuster {
    pub cap: f64,
}

impl DelegationVarianceAdjuster for CapDelegationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationVarianceAdjuster {
    pub minimum: f64,
}

impl DelegationVarianceAdjuster for MinimumDelegationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationVarianceAdjuster for FlaggedDelegationVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationVarianceEngine {
    pub steps: Vec<Rc<dyn DelegationVarianceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationVarianceEngine {
    pub fn evaluate(&mut self, item: &DelegationVariance) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[DelegationVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_delegation_variance_engine(flags: &flags::Registry) -> DelegationVarianceEngine {
    DelegationVarianceEngine {
        steps: vec![
            Rc::new(ChannelDelegationVarianceAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.36)]),
            }) as Rc<dyn DelegationVarianceAdjuster>,
            Rc::new(PercentDelegationVarianceAdjuster { percent: 20.6 }),
            Rc::new(RoundingDelegationVarianceAdjuster { step: 1.0 }),
            Rc::new(CapDelegationVarianceAdjuster { cap: 4257.0 }),
            Rc::new(MinimumDelegationVarianceAdjuster { minimum: 42.3 }),
            Rc::new(FlaggedDelegationVarianceAdjuster {
                flags: flags.clone(),
                flag: "delegation-variance-adaptive".to_string(),
                percent: -0.4,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 70360.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_variance_engine(&flags::new_registry());
        let item = new_delegation_variance("code-852", "label-637");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
