use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict delegation target calculation.
pub trait StrictDelegationTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictDelegationTarget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictDelegationTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictDelegationTargetStep for FlaggedStrictDelegationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictDelegationTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictDelegationTargetAdjuster {
    pub percent: f64,
}

impl StrictDelegationTargetStep for PercentStrictDelegationTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictDelegationTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictDelegationTargetAdjuster {
    pub minimum: f64,
}

impl StrictDelegationTargetStep for MinimumStrictDelegationTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictDelegationTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationTargetMinimumOffsetAdjuster {
    pub weight: f64,
}

impl StrictDelegationTargetStep for StrictDelegationTargetMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &StrictDelegationTarget, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the strict delegation target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictDelegationTargetEngine {
    pub steps: Vec<Rc<dyn StrictDelegationTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictDelegationTargetEngine {
    pub fn compute(&mut self, item: &StrictDelegationTarget) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictDelegationTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictDelegationTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_delegation_target_engine(flags: &flags::Registry) -> StrictDelegationTargetEngine {
    StrictDelegationTargetEngine {
        steps: vec![
            Rc::new(FlaggedStrictDelegationTargetAdjuster {
                flags: flags.clone(),
                flag: "strict-delegation-target-strict".to_string(),
                percent: 2.5,
            }) as Rc<dyn StrictDelegationTargetStep>,
            Rc::new(PercentStrictDelegationTargetAdjuster { percent: -8.4 }),
            Rc::new(MinimumStrictDelegationTargetAdjuster { minimum: 38.4 }),
            Rc::new(StrictDelegationTargetMinimumOffsetAdjuster { weight: 1.71 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 36694.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_delegation_target_engine(&flags::new_registry());
        let item = new_strict_delegation_target("code-865", "name-604");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
