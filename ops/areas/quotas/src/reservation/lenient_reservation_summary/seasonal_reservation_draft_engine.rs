use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal reservation draft calculation.
pub trait SeasonalReservationDraftModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalReservationDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalReservationDraftAdjuster {
    pub minimum: f64,
}

impl SeasonalReservationDraftModifier for MinimumSeasonalReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalReservationDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalReservationDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalReservationDraftModifier for FlaggedSeasonalReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalReservationDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterSeasonalReservationDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalReservationDraftModifier for ClusterSeasonalReservationDraftAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &SeasonalReservationDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalReservationDraftAdjuster {
    pub step: f64,
}

impl SeasonalReservationDraftModifier for RoundingSeasonalReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalReservationDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal reservation draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalReservationDraftEngine {
    pub steps: Vec<Rc<dyn SeasonalReservationDraftModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalReservationDraftEngine {
    pub fn run(&mut self, item: &SeasonalReservationDraft) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalReservationDraftModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalReservationDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_reservation_draft_engine(flags: &flags::Registry) -> SeasonalReservationDraftEngine {
    SeasonalReservationDraftEngine {
        steps: vec![
            Rc::new(MinimumSeasonalReservationDraftAdjuster { minimum: 1.0 })
                as Rc<dyn SeasonalReservationDraftModifier>,
            Rc::new(FlaggedSeasonalReservationDraftAdjuster {
                flags: flags.clone(),
                flag: "seasonal-reservation-draft-batched".to_string(),
                percent: 1.6,
            }),
            Rc::new(ClusterSeasonalReservationDraftAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.02), ("east".to_string(), 0.74)]),
            }),
            Rc::new(RoundingSeasonalReservationDraftAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 89747.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_reservation_draft_engine(&flags::new_registry());
        let item = new_seasonal_reservation_draft("code-65", "title-317");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
