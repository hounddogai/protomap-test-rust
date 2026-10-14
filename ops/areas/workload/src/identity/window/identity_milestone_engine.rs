use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity milestone calculation.
pub trait IdentityMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityMilestoneAdjuster for FlaggedIdentityMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitIdentityMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityMilestoneAdjuster for UnitIdentityMilestoneAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &IdentityMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIdentityMilestoneAdjuster {
    pub minimum: f64,
}

impl IdentityMilestoneAdjuster for MinimumIdentityMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IdentityMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityMilestoneAdjuster {
    pub step: f64,
}

impl IdentityMilestoneAdjuster for RoundingIdentityMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityMilestoneAdjuster for TieredIdentityMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the identity milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityMilestoneEngine {
    pub steps: Vec<Rc<dyn IdentityMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityMilestoneEngine {
    pub fn compute(&mut self, item: &IdentityMilestone) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityMilestoneAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_identity_milestone_engine(flags: &flags::Registry) -> IdentityMilestoneEngine {
    IdentityMilestoneEngine {
        steps: vec![
            Rc::new(FlaggedIdentityMilestoneAdjuster {
                flags: flags.clone(),
                flag: "identity-milestone-scheduled".to_string(),
                percent: 5.9,
            }) as Rc<dyn IdentityMilestoneAdjuster>,
            Rc::new(UnitIdentityMilestoneAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.96),
                    ("online".to_string(), 0.73),
                    ("retail".to_string(), 0.71),
                ]),
            }),
            Rc::new(MinimumIdentityMilestoneAdjuster { minimum: 40.0 }),
            Rc::new(RoundingIdentityMilestoneAdjuster { step: 0.25 }),
            Rc::new(TieredIdentityMilestoneAdjuster {
                bounds: vec![2212.0, 6479.0, 8503.0, 9108.0],
                rates: vec![1.03, 1.19, 0.96, 1.07],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 90012.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_identity_milestone_engine(&flags::new_registry());
        let item = new_identity_milestone("code-650", "label-104");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
