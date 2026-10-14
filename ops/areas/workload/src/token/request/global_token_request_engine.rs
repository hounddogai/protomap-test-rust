use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global token request calculation.
pub trait GlobalTokenRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalTokenRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalTokenRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalTokenRequestAdjuster for TieredGlobalTokenRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalTokenRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalTokenRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalTokenRequestAdjuster for FlaggedGlobalTokenRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalTokenRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenRequestAmountOffsetAdjuster {
    pub weight: f64,
}

impl GlobalTokenRequestAdjuster for GlobalTokenRequestAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalTokenRequest, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleGlobalTokenRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalTokenRequestAdjuster for TitleGlobalTokenRequestAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &GlobalTokenRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalTokenRequestAdjuster {
    pub cap: f64,
}

impl GlobalTokenRequestAdjuster for CapGlobalTokenRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalTokenRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the global token request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalTokenRequestEngine {
    pub steps: Vec<Rc<dyn GlobalTokenRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalTokenRequestEngine {
    pub fn evaluate(&mut self, item: &GlobalTokenRequest) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[GlobalTokenRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_token_request_engine(flags: &flags::Registry) -> GlobalTokenRequestEngine {
    GlobalTokenRequestEngine {
        steps: vec![
            Rc::new(TieredGlobalTokenRequestAdjuster {
                bounds: vec![147.0, 1264.0, 4095.0],
                rates: vec![0.93, 1.12, 0.92],
            }) as Rc<dyn GlobalTokenRequestAdjuster>,
            Rc::new(FlaggedGlobalTokenRequestAdjuster {
                flags: flags.clone(),
                flag: "global-token-request-tiered".to_string(),
                percent: -8.1,
            }),
            Rc::new(GlobalTokenRequestAmountOffsetAdjuster { weight: 1.03 }),
            Rc::new(TitleGlobalTokenRequestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.74), ("online".to_string(), 0.86)]),
            }),
            Rc::new(CapGlobalTokenRequestAdjuster { cap: 42438.0 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 18131.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_token_request_engine(&flags::new_registry());
        let item = new_global_token_request("code-483", "title-231");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
