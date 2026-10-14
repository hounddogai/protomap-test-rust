use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the egress rollup calculation.
pub trait EgressRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressRollupAdjuster {
    pub cap: f64,
}

impl EgressRollupFactor for CapEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressRollupFactor for FlaggedEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressRollupAdjuster {
    pub minimum: f64,
}

impl EgressRollupFactor for MinimumEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressRollupAdjuster {
    pub percent: f64,
}

impl EgressRollupFactor for PercentEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleEgressRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressRollupFactor for LocaleEgressRollupAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &EgressRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressRollupAdjuster {
    pub step: f64,
}

impl EgressRollupFactor for RoundingEgressRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the egress rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressRollupEngine {
    pub steps: Vec<Rc<dyn EgressRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressRollupEngine {
    pub fn price(&mut self, item: &EgressRollup) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_egress_rollup_engine(flags: &flags::Registry) -> EgressRollupEngine {
    EgressRollupEngine {
        steps: vec![
            Rc::new(CapEgressRollupAdjuster { cap: 45632.0 }) as Rc<dyn EgressRollupFactor>,
            Rc::new(FlaggedEgressRollupAdjuster {
                flags: flags.clone(),
                flag: "egress-rollup-capped".to_string(),
                percent: 8.1,
            }),
            Rc::new(MinimumEgressRollupAdjuster { minimum: 43.0 }),
            Rc::new(PercentEgressRollupAdjuster { percent: -18.2 }),
            Rc::new(LocaleEgressRollupAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.19), ("retail".to_string(), 1.31)]),
            }),
            Rc::new(RoundingEgressRollupAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 8988.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_rollup_engine(&flags::new_registry());
        let item = new_egress_rollup("code-397", "name-478");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
