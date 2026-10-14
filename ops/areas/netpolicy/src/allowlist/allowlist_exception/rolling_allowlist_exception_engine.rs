use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling allowlist exception calculation.
pub trait RollingAllowlistExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingAllowlistException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterRollingAllowlistExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingAllowlistExceptionModifier for ClusterRollingAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &RollingAllowlistException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingAllowlistExceptionAdjuster {
    pub step: f64,
}

impl RollingAllowlistExceptionModifier for RoundingRollingAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingAllowlistException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingAllowlistExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingAllowlistExceptionModifier for FlaggedRollingAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingAllowlistException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAllowlistExceptionAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl RollingAllowlistExceptionModifier for RollingAllowlistExceptionAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &RollingAllowlistException, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingAllowlistExceptionAdjuster {
    pub minimum: f64,
}

impl RollingAllowlistExceptionModifier for MinimumRollingAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingAllowlistException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingAllowlistExceptionAdjuster {
    pub percent: f64,
}

impl RollingAllowlistExceptionModifier for PercentRollingAllowlistExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingAllowlistException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling allowlist exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingAllowlistExceptionEngine {
    pub steps: Vec<Rc<dyn RollingAllowlistExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingAllowlistExceptionEngine {
    pub fn run(&mut self, item: &RollingAllowlistException) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollingAllowlistException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_allowlist_exception_engine(flags: &flags::Registry) -> RollingAllowlistExceptionEngine {
    RollingAllowlistExceptionEngine {
        steps: vec![
            Rc::new(ClusterRollingAllowlistExceptionAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.29)]),
            }) as Rc<dyn RollingAllowlistExceptionModifier>,
            Rc::new(RoundingRollingAllowlistExceptionAdjuster { step: 0.1 }),
            Rc::new(FlaggedRollingAllowlistExceptionAdjuster {
                flags: flags.clone(),
                flag: "rolling-allowlist-exception-weighted".to_string(),
                percent: -7.0,
            }),
            Rc::new(RollingAllowlistExceptionAttemptsOffsetAdjuster { weight: 1.54 }),
            Rc::new(MinimumRollingAllowlistExceptionAdjuster { minimum: 4.3 }),
            Rc::new(PercentRollingAllowlistExceptionAdjuster { percent: -15.2 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 47410.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_allowlist_exception_engine(&flags::new_registry());
        let item = new_rolling_allowlist_exception("code-14", "name-237");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
