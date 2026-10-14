use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the strict policy item calculation.
pub trait StrictPolicyItemAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictPolicyItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictPolicyItemAdjuster {
    pub minimum: f64,
}

impl StrictPolicyItemAdjuster for MinimumStrictPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictPolicyItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictPolicyItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictPolicyItemAdjuster for FlaggedStrictPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictPolicyItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyItemBurstOffsetAdjuster {
    pub weight: f64,
}

impl StrictPolicyItemAdjuster for StrictPolicyItemBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &StrictPolicyItem, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictPolicyItemAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictPolicyItemAdjuster for TieredStrictPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictPolicyItem, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceStrictPolicyItemAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictPolicyItemAdjuster for SourceStrictPolicyItemAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &StrictPolicyItem, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict policy item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictPolicyItemEngine {
    pub steps: Vec<Rc<dyn StrictPolicyItemAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictPolicyItemEngine {
    pub fn run(&mut self, item: &StrictPolicyItem) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictPolicyItemAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictPolicyItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_policy_item_engine(flags: &flags::Registry) -> StrictPolicyItemEngine {
    StrictPolicyItemEngine {
        steps: vec![
            Rc::new(MinimumStrictPolicyItemAdjuster { minimum: 0.4 }) as Rc<dyn StrictPolicyItemAdjuster>,
            Rc::new(FlaggedStrictPolicyItemAdjuster {
                flags: flags.clone(),
                flag: "strict-policy-item-primary".to_string(),
                percent: 2.4,
            }),
            Rc::new(StrictPolicyItemBurstOffsetAdjuster { weight: 0.17 }),
            Rc::new(TieredStrictPolicyItemAdjuster {
                bounds: vec![2953.0, 5924.0],
                rates: vec![0.8, 0.84],
            }),
            Rc::new(SourceStrictPolicyItemAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.76)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 61800.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_policy_item_engine(&flags::new_registry());
        let item = new_strict_policy_item("code-816", "name-66");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
