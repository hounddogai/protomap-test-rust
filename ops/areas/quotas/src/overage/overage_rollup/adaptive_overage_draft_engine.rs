use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive overage draft calculation.
pub trait AdaptiveOverageDraftFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveOverageDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveOverageDraftAdjuster {
    pub minimum: f64,
}

impl AdaptiveOverageDraftFactor for MinimumAdaptiveOverageDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveOverageDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAdaptiveOverageDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveOverageDraftFactor for TitleAdaptiveOverageDraftAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AdaptiveOverageDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveOverageDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveOverageDraftFactor for FlaggedAdaptiveOverageDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveOverageDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveOverageDraftAdjuster {
    pub step: f64,
}

impl AdaptiveOverageDraftFactor for RoundingAdaptiveOverageDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveOverageDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveOverageDraftAdjuster {
    pub percent: f64,
}

impl AdaptiveOverageDraftFactor for PercentAdaptiveOverageDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveOverageDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive overage draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveOverageDraftEngine {
    pub steps: Vec<Rc<dyn AdaptiveOverageDraftFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveOverageDraftEngine {
    pub fn run(&mut self, item: &AdaptiveOverageDraft) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveOverageDraftFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveOverageDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_overage_draft_engine(flags: &flags::Registry) -> AdaptiveOverageDraftEngine {
    AdaptiveOverageDraftEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveOverageDraftAdjuster { minimum: 38.5 }) as Rc<dyn AdaptiveOverageDraftFactor>,
            Rc::new(TitleAdaptiveOverageDraftAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.29), ("online".to_string(), 0.78)]),
            }),
            Rc::new(FlaggedAdaptiveOverageDraftAdjuster {
                flags: flags.clone(),
                flag: "adaptive-overage-draft-fallback".to_string(),
                percent: -5.3,
            }),
            Rc::new(RoundingAdaptiveOverageDraftAdjuster { step: 0.25 }),
            Rc::new(PercentAdaptiveOverageDraftAdjuster { percent: 10.8 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 78006.0,
    }
}
