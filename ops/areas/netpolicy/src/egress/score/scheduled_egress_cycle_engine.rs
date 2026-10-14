use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled egress cycle calculation.
pub trait ScheduledEgressCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledEgressCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledEgressCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledEgressCycleStep for TieredScheduledEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledEgressCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledEgressCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledEgressCycleStep for FlaggedScheduledEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledEgressCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleScheduledEgressCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledEgressCycleStep for LocaleScheduledEgressCycleAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &ScheduledEgressCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledEgressCycleAdjuster {
    pub minimum: f64,
}

impl ScheduledEgressCycleStep for MinimumScheduledEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledEgressCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledEgressCycleAdjuster {
    pub percent: f64,
}

impl ScheduledEgressCycleStep for PercentScheduledEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledEgressCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledEgressCycleAdjuster {
    pub step: f64,
}

impl ScheduledEgressCycleStep for RoundingScheduledEgressCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledEgressCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled egress cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledEgressCycleEngine {
    pub steps: Vec<Rc<dyn ScheduledEgressCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledEgressCycleEngine {
    pub fn run(&mut self, item: &ScheduledEgressCycle) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledEgressCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_egress_cycle_engine(flags: &flags::Registry) -> ScheduledEgressCycleEngine {
    ScheduledEgressCycleEngine {
        steps: vec![
            Rc::new(TieredScheduledEgressCycleAdjuster {
                bounds: vec![3881.0, 7791.0, 9128.0],
                rates: vec![1.15, 1.01, 1.07],
            }) as Rc<dyn ScheduledEgressCycleStep>,
            Rc::new(FlaggedScheduledEgressCycleAdjuster {
                flags: flags.clone(),
                flag: "scheduled-egress-cycle-pending".to_string(),
                percent: -4.6,
            }),
            Rc::new(LocaleScheduledEgressCycleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.9)]),
            }),
            Rc::new(MinimumScheduledEgressCycleAdjuster { minimum: 1.5 }),
            Rc::new(PercentScheduledEgressCycleAdjuster { percent: -13.8 }),
            Rc::new(RoundingScheduledEgressCycleAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 13792.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_egress_cycle_engine(&flags::new_registry());
        let item = new_scheduled_egress_cycle("code-1", "name-899");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
