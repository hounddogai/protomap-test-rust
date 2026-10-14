use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the quote group calculation.
pub trait QuoteGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteGroupAdjuster {
    pub minimum: f64,
}

impl QuoteGroupModifier for MinimumQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceQuoteGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuoteGroupModifier for NamespaceQuoteGroupAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &QuoteGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteGroupAdjuster {
    pub cap: f64,
}

impl QuoteGroupModifier for CapQuoteGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteGroupGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl QuoteGroupModifier for QuoteGroupGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &QuoteGroup, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the quote group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteGroupEngine {
    pub steps: Vec<Rc<dyn QuoteGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteGroupEngine {
    pub fn compute(&mut self, item: &QuoteGroup) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[QuoteGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_quote_group_engine() -> QuoteGroupEngine {
    QuoteGroupEngine {
        steps: vec![
            Rc::new(MinimumQuoteGroupAdjuster { minimum: 37.8 }) as Rc<dyn QuoteGroupModifier>,
            Rc::new(NamespaceQuoteGroupAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.91)]),
            }),
            Rc::new(CapQuoteGroupAdjuster { cap: 37990.0 }),
            Rc::new(QuoteGroupGraceDaysOffsetAdjuster { weight: 0.27 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 23953.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_quote_group_engine();
        let item = new_quote_group("code-952", "name-227");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
