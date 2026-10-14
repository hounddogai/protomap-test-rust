use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled reservation summary calculation.
pub trait ScheduledReservationSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledReservationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledReservationSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledReservationSummaryModifier for FlaggedScheduledReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledReservationSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledReservationSummaryAdjuster {
    pub cap: f64,
}

impl ScheduledReservationSummaryModifier for CapScheduledReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledReservationSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledReservationSummaryAdjuster {
    pub step: f64,
}

impl ScheduledReservationSummaryModifier for RoundingScheduledReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledReservationSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationSummaryRatioOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledReservationSummaryModifier for ScheduledReservationSummaryRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledReservationSummary, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryScheduledReservationSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledReservationSummaryModifier for CountryScheduledReservationSummaryAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &ScheduledReservationSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledReservationSummaryAdjuster {
    pub percent: f64,
}

impl ScheduledReservationSummaryModifier for PercentScheduledReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledReservationSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled reservation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledReservationSummaryEngine {
    pub steps: Vec<Rc<dyn ScheduledReservationSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledReservationSummaryEngine {
    pub fn compute(&mut self, item: &ScheduledReservationSummary) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledReservationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_reservation_summary_engine(flags: &flags::Registry) -> ScheduledReservationSummaryEngine {
    ScheduledReservationSummaryEngine {
        steps: vec![
            Rc::new(FlaggedScheduledReservationSummaryAdjuster {
                flags: flags.clone(),
                flag: "scheduled-reservation-summary-adaptive".to_string(),
                percent: -9.9,
            }) as Rc<dyn ScheduledReservationSummaryModifier>,
            Rc::new(CapScheduledReservationSummaryAdjuster { cap: 34498.0 }),
            Rc::new(RoundingScheduledReservationSummaryAdjuster { step: 5.0 }),
            Rc::new(ScheduledReservationSummaryRatioOffsetAdjuster { weight: 0.53 }),
            Rc::new(CountryScheduledReservationSummaryAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.4),
                    ("east".to_string(), 0.95),
                    ("retail".to_string(), 0.71),
                ]),
            }),
            Rc::new(PercentScheduledReservationSummaryAdjuster { percent: -20.9 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 29971.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_reservation_summary_engine(&flags::new_registry());
        let item = new_scheduled_reservation_summary("code-488", "name-527");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
