use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient delegation profile calculation.
pub trait LenientDelegationProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientDelegationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientDelegationProfileAdjuster {
    pub percent: f64,
}

impl LenientDelegationProfileModifier for PercentLenientDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientDelegationProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientDelegationProfileAdjuster {
    pub step: f64,
}

impl LenientDelegationProfileModifier for RoundingLenientDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientDelegationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientDelegationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientDelegationProfileModifier for FlaggedLenientDelegationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientDelegationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientDelegationProfileCeilingOffsetAdjuster {
    pub weight: f64,
}

impl LenientDelegationProfileModifier for LenientDelegationProfileCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &LenientDelegationProfile, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the lenient delegation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientDelegationProfileEngine {
    pub steps: Vec<Rc<dyn LenientDelegationProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientDelegationProfileEngine {
    pub fn compute(&mut self, item: &LenientDelegationProfile) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientDelegationProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientDelegationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_delegation_profile_engine(flags: &flags::Registry) -> LenientDelegationProfileEngine {
    LenientDelegationProfileEngine {
        steps: vec![
            Rc::new(PercentLenientDelegationProfileAdjuster { percent: 5.0 })
                as Rc<dyn LenientDelegationProfileModifier>,
            Rc::new(RoundingLenientDelegationProfileAdjuster { step: 0.1 }),
            Rc::new(FlaggedLenientDelegationProfileAdjuster {
                flags: flags.clone(),
                flag: "lenient-delegation-profile-strict".to_string(),
                percent: 2.8,
            }),
            Rc::new(LenientDelegationProfileCeilingOffsetAdjuster { weight: 1.92 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 88782.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_delegation_profile_engine(&flags::new_registry());
        let item = new_lenient_delegation_profile("code-857", "title-375");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
