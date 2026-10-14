use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the scheduled canary summary calculation.
pub trait ScheduledCanarySummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledCanarySummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledCanarySummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledCanarySummaryStep for TieredScheduledCanarySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledCanarySummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledCanarySummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledCanarySummaryStep for FlaggedScheduledCanarySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledCanarySummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledCanarySummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledCanarySummaryStep for NameScheduledCanarySummaryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledCanarySummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the scheduled canary summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledCanarySummaryEngine {
    pub steps: Vec<Rc<dyn ScheduledCanarySummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledCanarySummaryEngine {
    pub fn compute(&mut self, item: &ScheduledCanarySummary) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledCanarySummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledCanarySummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_canary_summary_engine(flags: &flags::Registry) -> ScheduledCanarySummaryEngine {
    ScheduledCanarySummaryEngine {
        steps: vec![
            Rc::new(TieredScheduledCanarySummaryAdjuster {
                bounds: vec![4025.0, 5447.0, 7242.0],
                rates: vec![0.97, 1.1, 0.92],
            }) as Rc<dyn ScheduledCanarySummaryStep>,
            Rc::new(FlaggedScheduledCanarySummaryAdjuster {
                flags: flags.clone(),
                flag: "scheduled-canary-summary-primary".to_string(),
                percent: -5.3,
            }),
            Rc::new(NameScheduledCanarySummaryAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.31)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 88677.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_canary_summary_engine(&flags::new_registry());
        let item = new_scheduled_canary_summary("code-556", "name-67");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
