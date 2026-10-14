use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped reservation milestone calculation.
pub trait CappedReservationMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedReservationMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCappedReservationMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedReservationMilestoneStep for NameCappedReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CappedReservationMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedReservationMilestoneAdjuster {
    pub percent: f64,
}

impl CappedReservationMilestoneStep for PercentCappedReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedReservationMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedReservationMilestoneAdjuster {
    pub cap: f64,
}

impl CappedReservationMilestoneStep for CapCappedReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedReservationMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedReservationMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedReservationMilestoneStep for FlaggedCappedReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedReservationMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedReservationMilestoneGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl CappedReservationMilestoneStep for CappedReservationMilestoneGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &CappedReservationMilestone, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedReservationMilestoneAdjuster {
    pub minimum: f64,
}

impl CappedReservationMilestoneStep for MinimumCappedReservationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedReservationMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped reservation milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedReservationMilestoneEngine {
    pub steps: Vec<Rc<dyn CappedReservationMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedReservationMilestoneEngine {
    pub fn run(&mut self, item: &CappedReservationMilestone) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedReservationMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_reservation_milestone_engine(flags: &flags::Registry) -> CappedReservationMilestoneEngine {
    CappedReservationMilestoneEngine {
        steps: vec![
            Rc::new(NameCappedReservationMilestoneAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.09), ("retail".to_string(), 0.74)]),
            }) as Rc<dyn CappedReservationMilestoneStep>,
            Rc::new(PercentCappedReservationMilestoneAdjuster { percent: 7.9 }),
            Rc::new(CapCappedReservationMilestoneAdjuster { cap: 20917.0 }),
            Rc::new(FlaggedCappedReservationMilestoneAdjuster {
                flags: flags.clone(),
                flag: "capped-reservation-milestone-adaptive".to_string(),
                percent: -3.7,
            }),
            Rc::new(CappedReservationMilestoneGraceDaysOffsetAdjuster { weight: 1.64 }),
            Rc::new(MinimumCappedReservationMilestoneAdjuster { minimum: 4.2 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 42770.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_reservation_milestone_engine(&flags::new_registry());
        let item = new_capped_reservation_milestone("code-207", "name-220");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
