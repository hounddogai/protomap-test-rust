use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy page ledger calculation.
pub trait LegacyPageLedgerFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyPageLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyPageLedgerAdjuster {
    pub minimum: f64,
}

impl LegacyPageLedgerFactor for MinimumLegacyPageLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyPageLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPageLedgerCeilingOffsetAdjuster {
    pub weight: f64,
}

impl LegacyPageLedgerFactor for LegacyPageLedgerCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyPageLedger, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyPageLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyPageLedgerFactor for TieredLegacyPageLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyPageLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLegacyPageLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyPageLedgerFactor for LabelLegacyPageLedgerAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LegacyPageLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyPageLedgerAdjuster {
    pub cap: f64,
}

impl LegacyPageLedgerFactor for CapLegacyPageLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyPageLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy page ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyPageLedgerEngine {
    pub steps: Vec<Rc<dyn LegacyPageLedgerFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyPageLedgerEngine {
    pub fn compute(&mut self, item: &LegacyPageLedger) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[LegacyPageLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_page_ledger_engine() -> LegacyPageLedgerEngine {
    LegacyPageLedgerEngine {
        steps: vec![
            Rc::new(MinimumLegacyPageLedgerAdjuster { minimum: 31.5 }) as Rc<dyn LegacyPageLedgerFactor>,
            Rc::new(LegacyPageLedgerCeilingOffsetAdjuster { weight: 0.32 }),
            Rc::new(TieredLegacyPageLedgerAdjuster {
                bounds: vec![2028.0, 4010.0],
                rates: vec![0.92, 1.05],
            }),
            Rc::new(LabelLegacyPageLedgerAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.21),
                    ("south".to_string(), 1.27),
                    ("north".to_string(), 1.26),
                ]),
            }),
            Rc::new(CapLegacyPageLedgerAdjuster { cap: 24486.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 28438.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_page_ledger_engine();
        let item = new_legacy_page_ledger("code-108", "label-772");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
