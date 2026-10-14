use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual delegation override calculation.
pub trait ManualDelegationOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDelegationOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualDelegationOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualDelegationOverrideAdjuster for TieredManualDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualDelegationOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDelegationOverrideRetriesOffsetAdjuster {
    pub weight: f64,
}

impl ManualDelegationOverrideAdjuster for ManualDelegationOverrideRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &ManualDelegationOverride, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualDelegationOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualDelegationOverrideAdjuster for FlaggedManualDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualDelegationOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual delegation override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDelegationOverrideEngine {
    pub steps: Vec<Rc<dyn ManualDelegationOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDelegationOverrideEngine {
    pub fn price(&mut self, item: &ManualDelegationOverride) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualDelegationOverrideAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualDelegationOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_delegation_override_engine(flags: &flags::Registry) -> ManualDelegationOverrideEngine {
    ManualDelegationOverrideEngine {
        steps: vec![
            Rc::new(TieredManualDelegationOverrideAdjuster {
                bounds: vec![6602.0, 9383.0],
                rates: vec![1.03, 1.07],
            }) as Rc<dyn ManualDelegationOverrideAdjuster>,
            Rc::new(ManualDelegationOverrideRetriesOffsetAdjuster { weight: 1.94 }),
            Rc::new(FlaggedManualDelegationOverrideAdjuster {
                flags: flags.clone(),
                flag: "manual-delegation-override-tiered".to_string(),
                percent: 3.1,
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 39418.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_delegation_override_engine(&flags::new_registry());
        let item = new_manual_delegation_override("code-137", "name-598");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
