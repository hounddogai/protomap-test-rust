use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist exception calculation.
pub trait AllowlistExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistExceptionAdjuster {
    pub cap: f64,
}

impl AllowlistExceptionModifier for CapAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistExceptionModifier for TieredAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistExceptionModifier for FlaggedAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAllowlistExceptionAdjuster {
    pub minimum: f64,
}

impl AllowlistExceptionModifier for MinimumAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AllowlistException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitAllowlistExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AllowlistExceptionModifier for UnitAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &AllowlistException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the allowlist exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistExceptionEngine {
    pub steps: Vec<Rc<dyn AllowlistExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistExceptionEngine {
    pub fn compute(&mut self, item: &AllowlistException) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_allowlist_exception_engine(flags: &flags::Registry) -> AllowlistExceptionEngine {
    AllowlistExceptionEngine {
        steps: vec![
            Rc::new(CapAllowlistExceptionAdjuster { cap: 29930.0 }) as Rc<dyn AllowlistExceptionModifier>,
            Rc::new(TieredAllowlistExceptionAdjuster {
                bounds: vec![7108.0, 9895.0],
                rates: vec![0.91, 1.09],
            }),
            Rc::new(FlaggedAllowlistExceptionAdjuster {
                flags: flags.clone(),
                flag: "allowlist-exception-primary".to_string(),
                percent: 2.0,
            }),
            Rc::new(MinimumAllowlistExceptionAdjuster { minimum: 34.3 }),
            Rc::new(UnitAllowlistExceptionAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.12),
                    ("central".to_string(), 1.19),
                    ("online".to_string(), 1.07),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 34283.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_allowlist_exception_engine(&flags::new_registry());
        let item = new_allowlist_exception("code-95", "name-409");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
