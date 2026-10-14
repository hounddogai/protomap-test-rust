use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the redaction window calculation.
pub trait RedactionWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RedactionWindow, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRedactionWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RedactionWindowModifier for FlaggedRedactionWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RedactionWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRedactionWindowAdjuster {
    pub step: f64,
}

impl RedactionWindowModifier for RoundingRedactionWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RedactionWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionWindowStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl RedactionWindowModifier for RedactionWindowStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &RedactionWindow, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRedactionWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RedactionWindowModifier for TieredRedactionWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RedactionWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRedactionWindowAdjuster {
    pub cap: f64,
}

impl RedactionWindowModifier for CapRedactionWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RedactionWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRedactionWindowAdjuster {
    pub percent: f64,
}

impl RedactionWindowModifier for PercentRedactionWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RedactionWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the redaction window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RedactionWindowEngine {
    pub steps: Vec<Rc<dyn RedactionWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RedactionWindowEngine {
    pub fn compute(&mut self, item: &RedactionWindow) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[RedactionWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_redaction_window_engine(flags: &flags::Registry) -> RedactionWindowEngine {
    RedactionWindowEngine {
        steps: vec![
            Rc::new(FlaggedRedactionWindowAdjuster {
                flags: flags.clone(),
                flag: "redaction-window-weighted".to_string(),
                percent: 0.6,
            }) as Rc<dyn RedactionWindowModifier>,
            Rc::new(RoundingRedactionWindowAdjuster { step: 1.0 }),
            Rc::new(RedactionWindowStartsAtOffsetAdjuster { weight: 0.15 }),
            Rc::new(TieredRedactionWindowAdjuster {
                bounds: vec![7126.0, 8213.0, 8317.0],
                rates: vec![0.89, 0.98, 0.9],
            }),
            Rc::new(CapRedactionWindowAdjuster { cap: 25580.0 }),
            Rc::new(PercentRedactionWindowAdjuster { percent: -14.7 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 49192.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_redaction_window_engine(&flags::new_registry());
        let item = new_redaction_window("code-628", "title-890");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
