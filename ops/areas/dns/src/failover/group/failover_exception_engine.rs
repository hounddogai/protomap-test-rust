use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover exception calculation.
pub trait FailoverExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverExceptionAdjuster for TieredFailoverExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeFailoverExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverExceptionAdjuster for ScopeFailoverExceptionAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &FailoverException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverExceptionAdjuster for FlaggedFailoverExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the failover exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverExceptionEngine {
    pub steps: Vec<Rc<dyn FailoverExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverExceptionEngine {
    pub fn evaluate(&mut self, item: &FailoverException) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[FailoverException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_failover_exception_engine(flags: &flags::Registry) -> FailoverExceptionEngine {
    FailoverExceptionEngine {
        steps: vec![
            Rc::new(TieredFailoverExceptionAdjuster {
                bounds: vec![1471.0, 4169.0, 7014.0, 9724.0],
                rates: vec![1.04, 0.97, 0.81, 1.0],
            }) as Rc<dyn FailoverExceptionAdjuster>,
            Rc::new(ScopeFailoverExceptionAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.09),
                    ("west".to_string(), 0.75),
                    ("north".to_string(), 0.73),
                ]),
            }),
            Rc::new(FlaggedFailoverExceptionAdjuster {
                flags: flags.clone(),
                flag: "failover-exception-primary".to_string(),
                percent: -4.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 94742.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_failover_exception_engine(&flags::new_registry());
        let item = new_failover_exception("code-146", "title-277");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
