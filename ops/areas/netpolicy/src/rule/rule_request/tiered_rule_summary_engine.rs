use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the tiered rule summary calculation.
pub trait TieredRuleSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredRuleSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredRuleSummaryAdjuster {
    pub cap: f64,
}

impl TieredRuleSummaryFactor for CapTieredRuleSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredRuleSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredRuleSummaryAdjuster {
    pub minimum: f64,
}

impl TieredRuleSummaryFactor for MinimumTieredRuleSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredRuleSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTieredRuleSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TieredRuleSummaryFactor for FlaggedTieredRuleSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TieredRuleSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredRuleSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredRuleSummaryFactor for TieredTieredRuleSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredRuleSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleSummaryCostOffsetAdjuster {
    pub weight: f64,
}

impl TieredRuleSummaryFactor for TieredRuleSummaryCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &TieredRuleSummary, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredRuleSummaryAdjuster {
    pub percent: f64,
}

impl TieredRuleSummaryFactor for PercentTieredRuleSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredRuleSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tiered rule summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredRuleSummaryEngine {
    pub steps: Vec<Rc<dyn TieredRuleSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredRuleSummaryEngine {
    pub fn evaluate(&mut self, item: &TieredRuleSummary) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredRuleSummaryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredRuleSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_rule_summary_engine(flags: &flags::Registry) -> TieredRuleSummaryEngine {
    TieredRuleSummaryEngine {
        steps: vec![
            Rc::new(CapTieredRuleSummaryAdjuster { cap: 35690.0 }) as Rc<dyn TieredRuleSummaryFactor>,
            Rc::new(MinimumTieredRuleSummaryAdjuster { minimum: 38.2 }),
            Rc::new(FlaggedTieredRuleSummaryAdjuster {
                flags: flags.clone(),
                flag: "tiered-rule-summary-primary".to_string(),
                percent: 6.3,
            }),
            Rc::new(TieredTieredRuleSummaryAdjuster {
                bounds: vec![1927.0, 3643.0, 5605.0, 7036.0],
                rates: vec![0.85, 1.15, 1.02, 1.09],
            }),
            Rc::new(TieredRuleSummaryCostOffsetAdjuster { weight: 0.42 }),
            Rc::new(PercentTieredRuleSummaryAdjuster { percent: 24.8 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 77933.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_rule_summary_engine(&flags::new_registry());
        let item = new_tiered_rule_summary("code-346", "title-11");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
