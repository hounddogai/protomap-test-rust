use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy postmortem plan calculation.
pub trait LegacyPostmortemPlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyPostmortemPlan, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyPostmortemPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyPostmortemPlanStep for FlaggedLegacyPostmortemPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyPostmortemPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyPostmortemPlanAdjuster {
    pub step: f64,
}

impl LegacyPostmortemPlanStep for RoundingLegacyPostmortemPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyPostmortemPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyPostmortemPlanAdjuster {
    pub percent: f64,
}

impl LegacyPostmortemPlanStep for PercentLegacyPostmortemPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyPostmortemPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyPostmortemPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyPostmortemPlanStep for TieredLegacyPostmortemPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyPostmortemPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the legacy postmortem plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyPostmortemPlanEngine {
    pub steps: Vec<Rc<dyn LegacyPostmortemPlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyPostmortemPlanEngine {
    pub fn compute(&mut self, item: &LegacyPostmortemPlan) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyPostmortemPlanStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyPostmortemPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_postmortem_plan_engine(flags: &flags::Registry) -> LegacyPostmortemPlanEngine {
    LegacyPostmortemPlanEngine {
        steps: vec![
            Rc::new(FlaggedLegacyPostmortemPlanAdjuster {
                flags: flags.clone(),
                flag: "legacy-postmortem-plan-manual".to_string(),
                percent: 4.0,
            }) as Rc<dyn LegacyPostmortemPlanStep>,
            Rc::new(RoundingLegacyPostmortemPlanAdjuster { step: 0.05 }),
            Rc::new(PercentLegacyPostmortemPlanAdjuster { percent: -14.5 }),
            Rc::new(TieredLegacyPostmortemPlanAdjuster {
                bounds: vec![184.0, 1442.0, 2341.0, 6482.0],
                rates: vec![1.06, 0.8, 0.83, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 60366.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_postmortem_plan_engine(&flags::new_registry());
        let item = new_legacy_postmortem_plan("code-249", "title-900");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
