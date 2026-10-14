use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending binding group calculation.
pub trait PendingBindingGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingBindingGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingBindingGroupAdjuster {
    pub minimum: f64,
}

impl PendingBindingGroupFactor for MinimumPendingBindingGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingBindingGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingGroupEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl PendingBindingGroupFactor for PendingBindingGroupEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &PendingBindingGroup, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPendingBindingGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingBindingGroupFactor for ChannelPendingBindingGroupAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PendingBindingGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingBindingGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingBindingGroupFactor for FlaggedPendingBindingGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingBindingGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingBindingGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingBindingGroupFactor for TieredPendingBindingGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingBindingGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingBindingGroupAdjuster {
    pub step: f64,
}

impl PendingBindingGroupFactor for RoundingPendingBindingGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingBindingGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the pending binding group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingBindingGroupEngine {
    pub steps: Vec<Rc<dyn PendingBindingGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingBindingGroupEngine {
    pub fn evaluate(&mut self, item: &PendingBindingGroup) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingBindingGroupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingBindingGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_binding_group_engine(flags: &flags::Registry) -> PendingBindingGroupEngine {
    PendingBindingGroupEngine {
        steps: vec![
            Rc::new(MinimumPendingBindingGroupAdjuster { minimum: 24.2 }) as Rc<dyn PendingBindingGroupFactor>,
            Rc::new(PendingBindingGroupEndsAtOffsetAdjuster { weight: 0.18 }),
            Rc::new(ChannelPendingBindingGroupAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.11), ("online".to_string(), 1.27)]),
            }),
            Rc::new(FlaggedPendingBindingGroupAdjuster {
                flags: flags.clone(),
                flag: "pending-binding-group-batched".to_string(),
                percent: -9.2,
            }),
            Rc::new(TieredPendingBindingGroupAdjuster {
                bounds: vec![4491.0, 7210.0],
                rates: vec![1.18, 1.1],
            }),
            Rc::new(RoundingPendingBindingGroupAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 13700.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_binding_group_engine(&flags::new_registry());
        let item = new_pending_binding_group("code-420", "name-315");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
