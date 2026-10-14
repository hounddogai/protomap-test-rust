use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal runbook draft calculation.
pub trait SeasonalRunbookDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalRunbookDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalRunbookDraftAdjuster {
    pub minimum: f64,
}

impl SeasonalRunbookDraftFactor for MinimumSeasonalRunbookDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalRunbookDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalRunbookDraftAdjuster {
    pub step: f64,
}

impl SeasonalRunbookDraftFactor for RoundingSeasonalRunbookDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalRunbookDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalRunbookDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalRunbookDraftFactor for FlaggedSeasonalRunbookDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalRunbookDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal runbook draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalRunbookDraftEngine {
    pub steps: Vec<Rc<dyn SeasonalRunbookDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalRunbookDraftEngine {
    pub fn price(&mut self, item: &SeasonalRunbookDraft) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalRunbookDraftFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalRunbookDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_runbook_draft_engine(flags: &flags::Registry) -> SeasonalRunbookDraftEngine {
    SeasonalRunbookDraftEngine {
        steps: vec![
            Rc::new(MinimumSeasonalRunbookDraftAdjuster { minimum: 7.3 }) as Rc<dyn SeasonalRunbookDraftFactor>,
            Rc::new(RoundingSeasonalRunbookDraftAdjuster { step: 0.25 }),
            Rc::new(FlaggedSeasonalRunbookDraftAdjuster {
                flags: flags.clone(),
                flag: "seasonal-runbook-draft-seasonal".to_string(),
                percent: 0.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 17033.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_runbook_draft_engine(&flags::new_registry());
        let item = new_seasonal_runbook_draft("code-38", "name-251");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
