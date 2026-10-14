use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive probe summary calculation.
pub trait AdaptiveProbeSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveProbeSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeSummarySequenceOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveProbeSummaryModifier for AdaptiveProbeSummarySequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveProbeSummary, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveProbeSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveProbeSummaryModifier for FlaggedAdaptiveProbeSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveProbeSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleAdaptiveProbeSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveProbeSummaryModifier for TitleAdaptiveProbeSummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &AdaptiveProbeSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveProbeSummaryAdjuster {
    pub minimum: f64,
}

impl AdaptiveProbeSummaryModifier for MinimumAdaptiveProbeSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveProbeSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveProbeSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveProbeSummaryModifier for TieredAdaptiveProbeSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveProbeSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveProbeSummaryAdjuster {
    pub percent: f64,
}

impl AdaptiveProbeSummaryModifier for PercentAdaptiveProbeSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveProbeSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive probe summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveProbeSummaryEngine {
    pub steps: Vec<Rc<dyn AdaptiveProbeSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveProbeSummaryEngine {
    pub fn price(&mut self, item: &AdaptiveProbeSummary) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveProbeSummaryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveProbeSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_probe_summary_engine(flags: &flags::Registry) -> AdaptiveProbeSummaryEngine {
    AdaptiveProbeSummaryEngine {
        steps: vec![
            Rc::new(AdaptiveProbeSummarySequenceOffsetAdjuster { weight: 1.95 })
                as Rc<dyn AdaptiveProbeSummaryModifier>,
            Rc::new(FlaggedAdaptiveProbeSummaryAdjuster {
                flags: flags.clone(),
                flag: "adaptive-probe-summary-adaptive".to_string(),
                percent: -7.2,
            }),
            Rc::new(TitleAdaptiveProbeSummaryAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.38), ("west".to_string(), 1.07)]),
            }),
            Rc::new(MinimumAdaptiveProbeSummaryAdjuster { minimum: 40.5 }),
            Rc::new(TieredAdaptiveProbeSummaryAdjuster {
                bounds: vec![4204.0, 5460.0],
                rates: vec![0.88, 1.12],
            }),
            Rc::new(PercentAdaptiveProbeSummaryAdjuster { percent: -23.2 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 10689.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_probe_summary_engine(&flags::new_registry());
        let item = new_adaptive_probe_summary("code-78", "title-26");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
