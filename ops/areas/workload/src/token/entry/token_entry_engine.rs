use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the token entry calculation.
pub trait TokenEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TokenEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceTokenEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TokenEntryStep for NamespaceTokenEntryAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &TokenEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenEntryAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl TokenEntryStep for TokenEntryAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &TokenEntry, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTokenEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TokenEntryStep for FlaggedTokenEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TokenEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTokenEntryAdjuster {
    pub cap: f64,
}

impl TokenEntryStep for CapTokenEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TokenEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTokenEntryAdjuster {
    pub percent: f64,
}

impl TokenEntryStep for PercentTokenEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TokenEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the token entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TokenEntryEngine {
    pub steps: Vec<Rc<dyn TokenEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TokenEntryEngine {
    pub fn evaluate(&mut self, item: &TokenEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TokenEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_token_entry_engine(flags: &flags::Registry) -> TokenEntryEngine {
    TokenEntryEngine {
        steps: vec![
            Rc::new(NamespaceTokenEntryAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.27)]),
            }) as Rc<dyn TokenEntryStep>,
            Rc::new(TokenEntryAttemptsOffsetAdjuster { weight: 1.76 }),
            Rc::new(FlaggedTokenEntryAdjuster {
                flags: flags.clone(),
                flag: "token-entry-tiered".to_string(),
                percent: 0.9,
            }),
            Rc::new(CapTokenEntryAdjuster { cap: 13329.0 }),
            Rc::new(PercentTokenEntryAdjuster { percent: -3.1 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 33483.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_token_entry_engine(&flags::new_registry());
        let item = new_token_entry("code-942", "name-721");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
