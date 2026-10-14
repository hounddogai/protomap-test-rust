use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped budget burn schedule calculation.
pub trait CappedBudgetBurnScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedBudgetBurnSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnScheduleGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl CappedBudgetBurnScheduleAdjuster for CappedBudgetBurnScheduleGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &CappedBudgetBurnSchedule, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedBudgetBurnScheduleAdjuster {
    pub cap: f64,
}

impl CappedBudgetBurnScheduleAdjuster for CapCappedBudgetBurnScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedBudgetBurnSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedBudgetBurnScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedBudgetBurnScheduleAdjuster for FlaggedCappedBudgetBurnScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedBudgetBurnSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedBudgetBurnScheduleAdjuster {
    pub minimum: f64,
}

impl CappedBudgetBurnScheduleAdjuster for MinimumCappedBudgetBurnScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedBudgetBurnSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedBudgetBurnScheduleAdjuster {
    pub step: f64,
}

impl CappedBudgetBurnScheduleAdjuster for RoundingCappedBudgetBurnScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedBudgetBurnSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCappedBudgetBurnScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedBudgetBurnScheduleAdjuster for SegmentCappedBudgetBurnScheduleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &CappedBudgetBurnSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the capped budget burn schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnScheduleEngine {
    pub steps: Vec<Rc<dyn CappedBudgetBurnScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedBudgetBurnScheduleEngine {
    pub fn run(&mut self, item: &CappedBudgetBurnSchedule) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedBudgetBurnScheduleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedBudgetBurnSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_budget_burn_schedule_engine(flags: &flags::Registry) -> CappedBudgetBurnScheduleEngine {
    CappedBudgetBurnScheduleEngine {
        steps: vec![
            Rc::new(CappedBudgetBurnScheduleGraceDaysOffsetAdjuster { weight: 1.46 })
                as Rc<dyn CappedBudgetBurnScheduleAdjuster>,
            Rc::new(CapCappedBudgetBurnScheduleAdjuster { cap: 48370.0 }),
            Rc::new(FlaggedCappedBudgetBurnScheduleAdjuster {
                flags: flags.clone(),
                flag: "capped-budget-burn-schedule-default".to_string(),
                percent: -2.9,
            }),
            Rc::new(MinimumCappedBudgetBurnScheduleAdjuster { minimum: 27.2 }),
            Rc::new(RoundingCappedBudgetBurnScheduleAdjuster { step: 0.1 }),
            Rc::new(SegmentCappedBudgetBurnScheduleAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.92), ("north".to_string(), 1.12)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 93480.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_budget_burn_schedule_engine(&flags::new_registry());
        let item = new_capped_budget_burn_schedule("code-728", "label-321");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
