use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient policy metric calculation.
pub trait LenientPolicyMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientPolicyMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientPolicyMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientPolicyMetricAdjuster for TieredLenientPolicyMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientPolicyMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientPolicyMetricAdjuster {
    pub minimum: f64,
}

impl LenientPolicyMetricAdjuster for MinimumLenientPolicyMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientPolicyMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelLenientPolicyMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientPolicyMetricAdjuster for ChannelLenientPolicyMetricAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &LenientPolicyMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyMetricAmountOffsetAdjuster {
    pub weight: f64,
}

impl LenientPolicyMetricAdjuster for LenientPolicyMetricAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &LenientPolicyMetric, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientPolicyMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientPolicyMetricAdjuster for FlaggedLenientPolicyMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientPolicyMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient policy metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientPolicyMetricEngine {
    pub steps: Vec<Rc<dyn LenientPolicyMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientPolicyMetricEngine {
    pub fn run(&mut self, item: &LenientPolicyMetric) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientPolicyMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientPolicyMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_policy_metric_engine(flags: &flags::Registry) -> LenientPolicyMetricEngine {
    LenientPolicyMetricEngine {
        steps: vec![
            Rc::new(TieredLenientPolicyMetricAdjuster {
                bounds: vec![734.0, 5380.0, 9068.0],
                rates: vec![1.06, 0.98, 0.82],
            }) as Rc<dyn LenientPolicyMetricAdjuster>,
            Rc::new(MinimumLenientPolicyMetricAdjuster { minimum: 24.5 }),
            Rc::new(ChannelLenientPolicyMetricAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.81), ("retail".to_string(), 0.98)]),
            }),
            Rc::new(LenientPolicyMetricAmountOffsetAdjuster { weight: 0.32 }),
            Rc::new(FlaggedLenientPolicyMetricAdjuster {
                flags: flags.clone(),
                flag: "lenient-policy-metric-batched".to_string(),
                percent: 5.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 12449.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_policy_metric_engine(&flags::new_registry());
        let item = new_lenient_policy_metric("code-15", "title-402");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
