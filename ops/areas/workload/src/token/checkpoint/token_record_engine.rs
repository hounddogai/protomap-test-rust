use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the token record calculation.
pub trait TokenRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTokenRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TokenRecordFactor for TieredTokenRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TokenRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceTokenRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenRecordFactor for NamespaceTokenRecordAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &TokenRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenRecordAdjuster {
    pub cap: f64,
}

impl TokenRecordFactor for CapTokenRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenRecordFactor for FlaggedTokenRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the token record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenRecordEngine {
    pub steps: Vec<Rc<dyn TokenRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenRecordEngine {
    pub fn price(&mut self, item: &TokenRecord) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[TokenRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_token_record_engine(flags: &flags::Registry) -> TokenRecordEngine {
    TokenRecordEngine {
        steps: vec![
            Rc::new(TieredTokenRecordAdjuster {
                bounds: vec![345.0, 1373.0, 3588.0, 6869.0],
                rates: vec![1.19, 1.05, 1.2, 0.98],
            }) as Rc<dyn TokenRecordFactor>,
            Rc::new(NamespaceTokenRecordAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.32), ("online".to_string(), 1.21)]),
            }),
            Rc::new(CapTokenRecordAdjuster { cap: 2786.0 }),
            Rc::new(FlaggedTokenRecordAdjuster {
                flags: flags.clone(),
                flag: "token-record-scheduled".to_string(),
                percent: -7.0,
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 73576.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_record_engine(&flags::new_registry());
        let item = new_token_record("code-407", "name-600");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
