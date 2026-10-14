use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary reservation summary calculation.
pub trait PrimaryReservationSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryReservationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPrimaryReservationSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryReservationSummaryFactor for LabelPrimaryReservationSummaryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PrimaryReservationSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryReservationSummaryAdjuster {
    pub step: f64,
}

impl PrimaryReservationSummaryFactor for RoundingPrimaryReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryReservationSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryReservationSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryReservationSummaryFactor for TieredPrimaryReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryReservationSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryReservationSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryReservationSummaryFactor for FlaggedPrimaryReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryReservationSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationSummaryDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryReservationSummaryFactor for PrimaryReservationSummaryDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryReservationSummary, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryReservationSummaryAdjuster {
    pub minimum: f64,
}

impl PrimaryReservationSummaryFactor for MinimumPrimaryReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryReservationSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary reservation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryReservationSummaryEngine {
    pub steps: Vec<Rc<dyn PrimaryReservationSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryReservationSummaryEngine {
    pub fn evaluate(&mut self, item: &PrimaryReservationSummary) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryReservationSummaryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryReservationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_reservation_summary_engine(flags: &flags::Registry) -> PrimaryReservationSummaryEngine {
    PrimaryReservationSummaryEngine {
        steps: vec![
            Rc::new(LabelPrimaryReservationSummaryAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.09),
                    ("retail".to_string(), 0.72),
                    ("central".to_string(), 1.26),
                ]),
            }) as Rc<dyn PrimaryReservationSummaryFactor>,
            Rc::new(RoundingPrimaryReservationSummaryAdjuster { step: 0.5 }),
            Rc::new(TieredPrimaryReservationSummaryAdjuster {
                bounds: vec![2293.0, 4583.0],
                rates: vec![0.95, 1.12],
            }),
            Rc::new(FlaggedPrimaryReservationSummaryAdjuster {
                flags: flags.clone(),
                flag: "primary-reservation-summary-primary".to_string(),
                percent: 1.4,
            }),
            Rc::new(PrimaryReservationSummaryDurationMinutesOffsetAdjuster { weight: 0.03 }),
            Rc::new(MinimumPrimaryReservationSummaryAdjuster { minimum: 32.8 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 61134.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_reservation_summary_engine(&flags::new_registry());
        let item = new_primary_reservation_summary("code-88", "label-63");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
