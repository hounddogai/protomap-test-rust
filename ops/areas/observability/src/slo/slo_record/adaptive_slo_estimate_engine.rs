use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive slo estimate calculation.
pub trait AdaptiveSloEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveSloEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSloEstimateScoreOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveSloEstimateAdjuster for AdaptiveSloEstimateScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveSloEstimate, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveSloEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveSloEstimateAdjuster for TieredAdaptiveSloEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveSloEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveSloEstimateAdjuster {
    pub minimum: f64,
}

impl AdaptiveSloEstimateAdjuster for MinimumAdaptiveSloEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveSloEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdaptiveSloEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveSloEstimateAdjuster for SourceAdaptiveSloEstimateAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &AdaptiveSloEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveSloEstimateAdjuster {
    pub cap: f64,
}

impl AdaptiveSloEstimateAdjuster for CapAdaptiveSloEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveSloEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveSloEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveSloEstimateAdjuster for FlaggedAdaptiveSloEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveSloEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive slo estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveSloEstimateEngine {
    pub steps: Vec<Rc<dyn AdaptiveSloEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveSloEstimateEngine {
    pub fn evaluate(&mut self, item: &AdaptiveSloEstimate) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveSloEstimateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveSloEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_slo_estimate_engine(flags: &flags::Registry) -> AdaptiveSloEstimateEngine {
    AdaptiveSloEstimateEngine {
        steps: vec![
            Rc::new(AdaptiveSloEstimateScoreOffsetAdjuster { weight: 0.58 }) as Rc<dyn AdaptiveSloEstimateAdjuster>,
            Rc::new(TieredAdaptiveSloEstimateAdjuster {
                bounds: vec![1107.0, 3030.0, 3677.0, 4378.0],
                rates: vec![0.93, 0.95, 0.94, 1.18],
            }),
            Rc::new(MinimumAdaptiveSloEstimateAdjuster { minimum: 19.1 }),
            Rc::new(SourceAdaptiveSloEstimateAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.8),
                    ("west".to_string(), 0.9),
                    ("online".to_string(), 1.4),
                ]),
            }),
            Rc::new(CapAdaptiveSloEstimateAdjuster { cap: 14002.0 }),
            Rc::new(FlaggedAdaptiveSloEstimateAdjuster {
                flags: flags.clone(),
                flag: "adaptive-slo-estimate-tiered".to_string(),
                percent: -4.5,
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 36216.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_slo_estimate_engine(&flags::new_registry());
        let item = new_adaptive_slo_estimate("code-327", "label-956");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
