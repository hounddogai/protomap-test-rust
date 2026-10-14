use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the regional policy item calculation.
pub trait RegionalPolicyItemFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPolicyItem, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalPolicyItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalPolicyItemFactor for FlaggedRegionalPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalPolicyItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalPolicyItemAdjuster {
    pub minimum: f64,
}

impl RegionalPolicyItemFactor for MinimumRegionalPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalPolicyItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyItemDiscountOffsetAdjuster {
    pub weight: f64,
}

impl RegionalPolicyItemFactor for RegionalPolicyItemDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalPolicyItem, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalPolicyItemAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalPolicyItemFactor for TieredRegionalPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalPolicyItem, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalPolicyItemAdjuster {
    pub percent: f64,
}

impl RegionalPolicyItemFactor for PercentRegionalPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalPolicyItem, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional policy item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPolicyItemEngine {
    pub steps: Vec<Rc<dyn RegionalPolicyItemFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPolicyItemEngine {
    pub fn run(&mut self, item: &RegionalPolicyItem) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalPolicyItemFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalPolicyItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_policy_item_engine(flags: &flags::Registry) -> RegionalPolicyItemEngine {
    RegionalPolicyItemEngine {
        steps: vec![
            Rc::new(FlaggedRegionalPolicyItemAdjuster {
                flags: flags.clone(),
                flag: "regional-policy-item-strict".to_string(),
                percent: 4.5,
            }) as Rc<dyn RegionalPolicyItemFactor>,
            Rc::new(MinimumRegionalPolicyItemAdjuster { minimum: 35.3 }),
            Rc::new(RegionalPolicyItemDiscountOffsetAdjuster { weight: 1.6 }),
            Rc::new(TieredRegionalPolicyItemAdjuster {
                bounds: vec![1059.0, 4725.0],
                rates: vec![0.94, 0.84],
            }),
            Rc::new(PercentRegionalPolicyItemAdjuster { percent: 15.9 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 86633.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_policy_item_engine(&flags::new_registry());
        let item = new_regional_policy_item("code-218", "name-765");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
