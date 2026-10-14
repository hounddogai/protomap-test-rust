use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled health check milestone calculation.
pub trait ScheduledHealthCheckMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledHealthCheckMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledHealthCheckMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledHealthCheckMilestoneStep for TieredScheduledHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledHealthCheckMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledHealthCheckMilestoneAdjuster {
    pub minimum: f64,
}

impl ScheduledHealthCheckMilestoneStep for MinimumScheduledHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledHealthCheckMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledHealthCheckMilestoneQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledHealthCheckMilestoneStep for ScheduledHealthCheckMilestoneQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledHealthCheckMilestone, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledHealthCheckMilestoneAdjuster {
    pub percent: f64,
}

impl ScheduledHealthCheckMilestoneStep for PercentScheduledHealthCheckMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledHealthCheckMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled health check milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledHealthCheckMilestoneEngine {
    pub steps: Vec<Rc<dyn ScheduledHealthCheckMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledHealthCheckMilestoneEngine {
    pub fn run(&mut self, item: &ScheduledHealthCheckMilestone) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledHealthCheckMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledHealthCheckMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_health_check_milestone_engine() -> ScheduledHealthCheckMilestoneEngine {
    ScheduledHealthCheckMilestoneEngine {
        steps: vec![
            Rc::new(TieredScheduledHealthCheckMilestoneAdjuster {
                bounds: vec![1483.0, 2219.0, 7055.0],
                rates: vec![1.08, 0.83, 1.0],
            }) as Rc<dyn ScheduledHealthCheckMilestoneStep>,
            Rc::new(MinimumScheduledHealthCheckMilestoneAdjuster { minimum: 27.9 }),
            Rc::new(ScheduledHealthCheckMilestoneQuantityOffsetAdjuster { weight: 1.48 }),
            Rc::new(PercentScheduledHealthCheckMilestoneAdjuster { percent: 9.3 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 94304.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_health_check_milestone_engine();
        let item = new_scheduled_health_check_milestone("code-741", "title-208");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
