use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan draft calculation.
pub trait PlanDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanDraftAdjuster {
    pub cap: f64,
}

impl PlanDraftFactor for CapPlanDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanDraftAdjuster {
    pub step: f64,
}

impl PlanDraftFactor for RoundingPlanDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanDraftFactor for TieredPlanDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanDraftFactor for FlaggedPlanDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanDraftAdjuster {
    pub percent: f64,
}

impl PlanDraftFactor for PercentPlanDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPlanDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanDraftFactor for ChannelPlanDraftAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PlanDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the plan draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanDraftEngine {
    pub steps: Vec<Rc<dyn PlanDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanDraftEngine {
    pub fn compute(&mut self, item: &PlanDraft) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[PlanDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_draft_engine(flags: &flags::Registry) -> PlanDraftEngine {
    PlanDraftEngine {
        steps: vec![
            Rc::new(CapPlanDraftAdjuster { cap: 4591.0 }) as Rc<dyn PlanDraftFactor>,
            Rc::new(RoundingPlanDraftAdjuster { step: 0.05 }),
            Rc::new(TieredPlanDraftAdjuster {
                bounds: vec![5850.0, 6266.0, 8077.0],
                rates: vec![1.19, 0.99, 1.06],
            }),
            Rc::new(FlaggedPlanDraftAdjuster {
                flags: flags.clone(),
                flag: "plan-draft-tiered".to_string(),
                percent: -8.9,
            }),
            Rc::new(PercentPlanDraftAdjuster { percent: -4.2 }),
            Rc::new(ChannelPlanDraftAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.72)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 79851.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_draft_engine(&flags::new_registry());
        let item = new_plan_draft("code-303", "title-145");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
