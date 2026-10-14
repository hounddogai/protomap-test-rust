use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive overage milestone calculation.
pub trait AdaptiveOverageMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveOverageMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelAdaptiveOverageMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveOverageMilestoneFactor for ChannelAdaptiveOverageMilestoneAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &AdaptiveOverageMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveOverageMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveOverageMilestoneFactor for TieredAdaptiveOverageMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveOverageMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveOverageMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveOverageMilestoneFactor for FlaggedAdaptiveOverageMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveOverageMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveOverageMilestoneAdjuster {
    pub cap: f64,
}

impl AdaptiveOverageMilestoneFactor for CapAdaptiveOverageMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveOverageMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveOverageMilestoneAdjuster {
    pub step: f64,
}

impl AdaptiveOverageMilestoneFactor for RoundingAdaptiveOverageMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveOverageMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive overage milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveOverageMilestoneEngine {
    pub steps: Vec<Rc<dyn AdaptiveOverageMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveOverageMilestoneEngine {
    pub fn evaluate(&mut self, item: &AdaptiveOverageMilestone) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveOverageMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_overage_milestone_engine(flags: &flags::Registry) -> AdaptiveOverageMilestoneEngine {
    AdaptiveOverageMilestoneEngine {
        steps: vec![
            Rc::new(ChannelAdaptiveOverageMilestoneAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.21), ("east".to_string(), 0.93)]),
            }) as Rc<dyn AdaptiveOverageMilestoneFactor>,
            Rc::new(TieredAdaptiveOverageMilestoneAdjuster {
                bounds: vec![8050.0, 8280.0, 9694.0],
                rates: vec![1.12, 1.07, 0.99],
            }),
            Rc::new(FlaggedAdaptiveOverageMilestoneAdjuster {
                flags: flags.clone(),
                flag: "adaptive-overage-milestone-strict".to_string(),
                percent: -9.3,
            }),
            Rc::new(CapAdaptiveOverageMilestoneAdjuster { cap: 6997.0 }),
            Rc::new(RoundingAdaptiveOverageMilestoneAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 11501.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_overage_milestone_engine(&flags::new_registry());
        let item = new_adaptive_overage_milestone("code-771", "label-64");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
