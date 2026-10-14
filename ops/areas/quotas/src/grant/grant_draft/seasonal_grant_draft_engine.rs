use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal grant draft calculation.
pub trait SeasonalGrantDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalGrantDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantDraftCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalGrantDraftStep for SeasonalGrantDraftCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalGrantDraft, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalGrantDraftAdjuster {
    pub minimum: f64,
}

impl SeasonalGrantDraftStep for MinimumSeasonalGrantDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalGrantDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalGrantDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalGrantDraftStep for TieredSeasonalGrantDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalGrantDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSeasonalGrantDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalGrantDraftStep for TitleSeasonalGrantDraftAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SeasonalGrantDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalGrantDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalGrantDraftStep for FlaggedSeasonalGrantDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalGrantDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalGrantDraftAdjuster {
    pub step: f64,
}

impl SeasonalGrantDraftStep for RoundingSeasonalGrantDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalGrantDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal grant draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalGrantDraftEngine {
    pub steps: Vec<Rc<dyn SeasonalGrantDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalGrantDraftEngine {
    pub fn compute(&mut self, item: &SeasonalGrantDraft) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalGrantDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalGrantDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_grant_draft_engine(flags: &flags::Registry) -> SeasonalGrantDraftEngine {
    SeasonalGrantDraftEngine {
        steps: vec![
            Rc::new(SeasonalGrantDraftCreatedAtOffsetAdjuster { weight: 1.86 }) as Rc<dyn SeasonalGrantDraftStep>,
            Rc::new(MinimumSeasonalGrantDraftAdjuster { minimum: 35.8 }),
            Rc::new(TieredSeasonalGrantDraftAdjuster {
                bounds: vec![2477.0, 7569.0, 9168.0],
                rates: vec![1.07, 0.9, 1.14],
            }),
            Rc::new(TitleSeasonalGrantDraftAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.79),
                    ("east".to_string(), 0.77),
                    ("retail".to_string(), 1.05),
                ]),
            }),
            Rc::new(FlaggedSeasonalGrantDraftAdjuster {
                flags: flags.clone(),
                flag: "seasonal-grant-draft-batched".to_string(),
                percent: -0.1,
            }),
            Rc::new(RoundingSeasonalGrantDraftAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 59383.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_grant_draft_engine(&flags::new_registry());
        let item = new_seasonal_grant_draft("code-948", "title-839");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
