use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence policy calculation.
pub trait SilencePolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilencePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilencePolicyAdjuster {
    pub minimum: f64,
}

impl SilencePolicyFactor for MinimumSilencePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilencePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilencePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilencePolicyFactor for TieredSilencePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilencePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilencePolicyAdjuster {
    pub percent: f64,
}

impl SilencePolicyFactor for PercentSilencePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilencePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceSilencePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilencePolicyFactor for NamespaceSilencePolicyAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &SilencePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilencePolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilencePolicyFactor for FlaggedSilencePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilencePolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the silence policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilencePolicyEngine {
    pub steps: Vec<Rc<dyn SilencePolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilencePolicyEngine {
    pub fn evaluate(&mut self, item: &SilencePolicy) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SilencePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_silence_policy_engine(flags: &flags::Registry) -> SilencePolicyEngine {
    SilencePolicyEngine {
        steps: vec![
            Rc::new(MinimumSilencePolicyAdjuster { minimum: 44.8 }) as Rc<dyn SilencePolicyFactor>,
            Rc::new(TieredSilencePolicyAdjuster {
                bounds: vec![111.0, 2839.0, 3722.0],
                rates: vec![1.03, 1.0, 1.02],
            }),
            Rc::new(PercentSilencePolicyAdjuster { percent: -19.9 }),
            Rc::new(NamespaceSilencePolicyAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.32), ("west".to_string(), 1.01)]),
            }),
            Rc::new(FlaggedSilencePolicyAdjuster {
                flags: flags.clone(),
                flag: "silence-policy-adaptive".to_string(),
                percent: 0.5,
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 47925.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_silence_policy_engine(&flags::new_registry());
        let item = new_silence_policy("code-756", "name-88");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
