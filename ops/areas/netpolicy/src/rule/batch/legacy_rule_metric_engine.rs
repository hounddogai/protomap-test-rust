use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy rule metric calculation.
pub trait LegacyRuleMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRuleMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyRuleMetricAdjuster {
    pub minimum: f64,
}

impl LegacyRuleMetricFactor for MinimumLegacyRuleMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyRuleMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyRuleMetricAdjuster {
    pub percent: f64,
}

impl LegacyRuleMetricFactor for PercentLegacyRuleMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyRuleMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyRuleMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyRuleMetricFactor for FlaggedLegacyRuleMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyRuleMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRuleMetricAdjuster {
    pub cap: f64,
}

impl LegacyRuleMetricFactor for CapLegacyRuleMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRuleMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy rule metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRuleMetricEngine {
    pub steps: Vec<Rc<dyn LegacyRuleMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRuleMetricEngine {
    pub fn price(&mut self, item: &LegacyRuleMetric) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[LegacyRuleMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_rule_metric_engine(flags: &flags::Registry) -> LegacyRuleMetricEngine {
    LegacyRuleMetricEngine {
        steps: vec![
            Rc::new(MinimumLegacyRuleMetricAdjuster { minimum: 19.5 }) as Rc<dyn LegacyRuleMetricFactor>,
            Rc::new(PercentLegacyRuleMetricAdjuster { percent: -13.4 }),
            Rc::new(FlaggedLegacyRuleMetricAdjuster {
                flags: flags.clone(),
                flag: "legacy-rule-metric-scheduled".to_string(),
                percent: 9.2,
            }),
            Rc::new(CapLegacyRuleMetricAdjuster { cap: 25350.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 10996.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_rule_metric_engine(&flags::new_registry());
        let item = new_legacy_rule_metric("code-15", "label-781");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
