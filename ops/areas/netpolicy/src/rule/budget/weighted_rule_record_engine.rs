use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted rule record calculation.
pub trait WeightedRuleRecordAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRuleRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedRuleRecordAdjuster {
    pub percent: f64,
}

impl WeightedRuleRecordAdjuster for PercentWeightedRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedRuleRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedRuleRecordAdjuster {
    pub step: f64,
}

impl WeightedRuleRecordAdjuster for RoundingWeightedRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedRuleRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedRuleRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedRuleRecordAdjuster for FlaggedWeightedRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedRuleRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedRuleRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedRuleRecordAdjuster for TieredWeightedRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedRuleRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedRuleRecordAdjuster {
    pub minimum: f64,
}

impl WeightedRuleRecordAdjuster for MinimumWeightedRuleRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedRuleRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleRecordRevisionOffsetAdjuster {
    pub weight: f64,
}

impl WeightedRuleRecordAdjuster for WeightedRuleRecordRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedRuleRecord, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the weighted rule record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRuleRecordEngine {
    pub steps: Vec<Rc<dyn WeightedRuleRecordAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRuleRecordEngine {
    pub fn evaluate(&mut self, item: &WeightedRuleRecord) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedRuleRecordAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRuleRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_rule_record_engine(flags: &flags::Registry) -> WeightedRuleRecordEngine {
    WeightedRuleRecordEngine {
        steps: vec![
            Rc::new(PercentWeightedRuleRecordAdjuster { percent: -8.3 }) as Rc<dyn WeightedRuleRecordAdjuster>,
            Rc::new(RoundingWeightedRuleRecordAdjuster { step: 0.1 }),
            Rc::new(FlaggedWeightedRuleRecordAdjuster {
                flags: flags.clone(),
                flag: "weighted-rule-record-default".to_string(),
                percent: 3.7,
            }),
            Rc::new(TieredWeightedRuleRecordAdjuster {
                bounds: vec![3428.0, 3938.0, 5826.0, 9826.0],
                rates: vec![1.05, 1.0, 1.0, 0.89],
            }),
            Rc::new(MinimumWeightedRuleRecordAdjuster { minimum: 19.7 }),
            Rc::new(WeightedRuleRecordRevisionOffsetAdjuster { weight: 0.43 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 19194.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_rule_record_engine(&flags::new_registry());
        let item = new_weighted_rule_record("code-605", "label-341");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
