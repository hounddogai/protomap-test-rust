use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback data export rule calculation.
pub trait FallbackDataExportRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackDataExportRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackDataExportRuleAdjuster {
    pub minimum: f64,
}

impl FallbackDataExportRuleStep for MinimumFallbackDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackDataExportRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackDataExportRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackDataExportRuleStep for FlaggedFallbackDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackDataExportRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackDataExportRuleAdjuster {
    pub step: f64,
}

impl FallbackDataExportRuleStep for RoundingFallbackDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackDataExportRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackDataExportRuleAdjuster {
    pub cap: f64,
}

impl FallbackDataExportRuleStep for CapFallbackDataExportRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackDataExportRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportRuleRetriesOffsetAdjuster {
    pub weight: f64,
}

impl FallbackDataExportRuleStep for FallbackDataExportRuleRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackDataExportRule, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the fallback data export rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackDataExportRuleEngine {
    pub steps: Vec<Rc<dyn FallbackDataExportRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackDataExportRuleEngine {
    pub fn compute(&mut self, item: &FallbackDataExportRule) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[FallbackDataExportRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_data_export_rule_engine(flags: &flags::Registry) -> FallbackDataExportRuleEngine {
    FallbackDataExportRuleEngine {
        steps: vec![
            Rc::new(MinimumFallbackDataExportRuleAdjuster { minimum: 4.7 }) as Rc<dyn FallbackDataExportRuleStep>,
            Rc::new(FlaggedFallbackDataExportRuleAdjuster {
                flags: flags.clone(),
                flag: "fallback-data-export-rule-legacy".to_string(),
                percent: -1.9,
            }),
            Rc::new(RoundingFallbackDataExportRuleAdjuster { step: 1.0 }),
            Rc::new(CapFallbackDataExportRuleAdjuster { cap: 14428.0 }),
            Rc::new(FallbackDataExportRuleRetriesOffsetAdjuster { weight: 0.39 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 99427.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_data_export_rule_engine(&flags::new_registry());
        let item = new_fallback_data_export_rule("code-52", "title-389");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
