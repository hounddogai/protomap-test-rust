use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the egress profile calculation.
pub trait EgressProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressProfileModifier for TieredEgressProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressProfileModifier for FlaggedEgressProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressProfileAdjuster {
    pub cap: f64,
}

impl EgressProfileModifier for CapEgressProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the egress profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressProfileEngine {
    pub steps: Vec<Rc<dyn EgressProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressProfileEngine {
    pub fn run(&mut self, item: &EgressProfile) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_egress_profile_engine(flags: &flags::Registry) -> EgressProfileEngine {
    EgressProfileEngine {
        steps: vec![
            Rc::new(TieredEgressProfileAdjuster {
                bounds: vec![2013.0, 3098.0, 5605.0, 6079.0],
                rates: vec![1.17, 0.83, 1.04, 0.87],
            }) as Rc<dyn EgressProfileModifier>,
            Rc::new(FlaggedEgressProfileAdjuster {
                flags: flags.clone(),
                flag: "egress-profile-pending".to_string(),
                percent: 9.8,
            }),
            Rc::new(CapEgressProfileAdjuster { cap: 46552.0 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 85372.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_profile_engine(&flags::new_registry());
        let item = new_egress_profile("code-216", "label-656");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
