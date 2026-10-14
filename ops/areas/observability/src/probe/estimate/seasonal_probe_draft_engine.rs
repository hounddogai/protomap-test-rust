use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal probe draft calculation.
pub trait SeasonalProbeDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalProbeDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalProbeDraftAdjuster {
    pub minimum: f64,
}

impl SeasonalProbeDraftStep for MinimumSeasonalProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalProbeDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalProbeDraftAdjuster {
    pub percent: f64,
}

impl SeasonalProbeDraftStep for PercentSeasonalProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalProbeDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSeasonalProbeDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalProbeDraftStep for LabelSeasonalProbeDraftAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SeasonalProbeDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalProbeDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalProbeDraftStep for FlaggedSeasonalProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalProbeDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalProbeDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalProbeDraftStep for TieredSeasonalProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalProbeDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal probe draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalProbeDraftEngine {
    pub steps: Vec<Rc<dyn SeasonalProbeDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalProbeDraftEngine {
    pub fn run(&mut self, item: &SeasonalProbeDraft) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalProbeDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_probe_draft_engine(flags: &flags::Registry) -> SeasonalProbeDraftEngine {
    SeasonalProbeDraftEngine {
        steps: vec![
            Rc::new(MinimumSeasonalProbeDraftAdjuster { minimum: 10.1 }) as Rc<dyn SeasonalProbeDraftStep>,
            Rc::new(PercentSeasonalProbeDraftAdjuster { percent: 18.9 }),
            Rc::new(LabelSeasonalProbeDraftAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.15),
                    ("west".to_string(), 1.05),
                    ("north".to_string(), 1.18),
                ]),
            }),
            Rc::new(FlaggedSeasonalProbeDraftAdjuster {
                flags: flags.clone(),
                flag: "seasonal-probe-draft-strict".to_string(),
                percent: 6.4,
            }),
            Rc::new(TieredSeasonalProbeDraftAdjuster {
                bounds: vec![1720.0, 4372.0, 7287.0],
                rates: vec![1.04, 1.02, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 43247.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_probe_draft_engine(&flags::new_registry());
        let item = new_seasonal_probe_draft("code-549", "label-772");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
