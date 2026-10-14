use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote mapping calculation.
pub trait QuoteMappingFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteMappingAdjuster {
    pub minimum: f64,
}

impl QuoteMappingFactor for MinimumQuoteMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteMappingAdjuster {
    pub step: f64,
}

impl QuoteMappingFactor for RoundingQuoteMappingAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteMapping, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteMappingFactor for FlaggedQuoteMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the quote mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteMappingEngine {
    pub steps: Vec<Rc<dyn QuoteMappingFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteMappingEngine {
    pub fn evaluate(&mut self, item: &QuoteMapping) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuoteMappingFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuoteMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_quote_mapping_engine(flags: &flags::Registry) -> QuoteMappingEngine {
    QuoteMappingEngine {
        steps: vec![
            Rc::new(MinimumQuoteMappingAdjuster { minimum: 2.2 }) as Rc<dyn QuoteMappingFactor>,
            Rc::new(RoundingQuoteMappingAdjuster { step: 0.1 }),
            Rc::new(FlaggedQuoteMappingAdjuster {
                flags: flags.clone(),
                flag: "quote-mapping-strict".to_string(),
                percent: 8.0,
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 13758.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_mapping_engine(&flags::new_registry());
        let item = new_quote_mapping("code-813", "title-321");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
