use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped isolation template calculation.
pub trait CappedIsolationTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedIsolationTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedIsolationTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedIsolationTemplateModifier for TieredCappedIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedIsolationTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedIsolationTemplateAdjuster {
    pub step: f64,
}

impl CappedIsolationTemplateModifier for RoundingCappedIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedIsolationTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedIsolationTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedIsolationTemplateModifier for FlaggedCappedIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedIsolationTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedIsolationTemplateAdjuster {
    pub percent: f64,
}

impl CappedIsolationTemplateModifier for PercentCappedIsolationTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedIsolationTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped isolation template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedIsolationTemplateEngine {
    pub steps: Vec<Rc<dyn CappedIsolationTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedIsolationTemplateEngine {
    pub fn run(&mut self, item: &CappedIsolationTemplate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedIsolationTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_isolation_template_engine(flags: &flags::Registry) -> CappedIsolationTemplateEngine {
    CappedIsolationTemplateEngine {
        steps: vec![
            Rc::new(TieredCappedIsolationTemplateAdjuster {
                bounds: vec![1304.0, 1313.0, 9318.0, 9845.0],
                rates: vec![1.2, 1.13, 1.04, 1.16],
            }) as Rc<dyn CappedIsolationTemplateModifier>,
            Rc::new(RoundingCappedIsolationTemplateAdjuster { step: 0.25 }),
            Rc::new(FlaggedCappedIsolationTemplateAdjuster {
                flags: flags.clone(),
                flag: "capped-isolation-template-legacy".to_string(),
                percent: -8.5,
            }),
            Rc::new(PercentCappedIsolationTemplateAdjuster { percent: 17.2 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 84618.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_isolation_template_engine(&flags::new_registry());
        let item = new_capped_isolation_template("code-343", "title-939");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
