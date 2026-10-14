use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce milestone calculation.
pub trait NonceMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceMilestoneAdjuster {
    pub step: f64,
}

impl NonceMilestoneFactor for RoundingNonceMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceMilestoneFactor for FlaggedNonceMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceMilestoneEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl NonceMilestoneFactor for NonceMilestoneEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &NonceMilestone, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

/// Runs the nonce milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceMilestoneEngine {
    pub steps: Vec<Rc<dyn NonceMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceMilestoneEngine {
    pub fn compute(&mut self, item: &NonceMilestone) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[NonceMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_nonce_milestone_engine(flags: &flags::Registry) -> NonceMilestoneEngine {
    NonceMilestoneEngine {
        steps: vec![
            Rc::new(RoundingNonceMilestoneAdjuster { step: 0.5 }) as Rc<dyn NonceMilestoneFactor>,
            Rc::new(FlaggedNonceMilestoneAdjuster {
                flags: flags.clone(),
                flag: "nonce-milestone-seasonal".to_string(),
                percent: -9.6,
            }),
            Rc::new(NonceMilestoneEndsAtOffsetAdjuster { weight: 0.28 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 80585.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_milestone_engine(&flags::new_registry());
        let item = new_nonce_milestone("code-586", "title-323");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
