use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the probe policy calculation.
pub trait ProbePolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbePolicyAdjuster {
    pub percent: f64,
}

impl ProbePolicyAdjuster for PercentProbePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbePolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbePolicyAdjuster for FlaggedProbePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbePolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbePolicyPriorityOffsetAdjuster {
    pub weight: f64,
}

impl ProbePolicyAdjuster for ProbePolicyPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &ProbePolicy, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelProbePolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbePolicyAdjuster for ChannelProbePolicyAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &ProbePolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbePolicyAdjuster for TieredProbePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbePolicyAdjuster {
    pub cap: f64,
}

impl ProbePolicyAdjuster for CapProbePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the probe policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbePolicyEngine {
    pub steps: Vec<Rc<dyn ProbePolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbePolicyEngine {
    pub fn compute(&mut self, item: &ProbePolicy) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ProbePolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ProbePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_probe_policy_engine(flags: &flags::Registry) -> ProbePolicyEngine {
    ProbePolicyEngine {
        steps: vec![
            Rc::new(PercentProbePolicyAdjuster { percent: 24.9 }) as Rc<dyn ProbePolicyAdjuster>,
            Rc::new(FlaggedProbePolicyAdjuster {
                flags: flags.clone(),
                flag: "probe-policy-lenient".to_string(),
                percent: 4.6,
            }),
            Rc::new(ProbePolicyPriorityOffsetAdjuster { weight: 1.66 }),
            Rc::new(ChannelProbePolicyAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.35), ("central".to_string(), 0.73)]),
            }),
            Rc::new(TieredProbePolicyAdjuster {
                bounds: vec![4405.0, 5691.0, 6048.0, 9303.0],
                rates: vec![0.82, 0.99, 1.12, 0.84],
            }),
            Rc::new(CapProbePolicyAdjuster { cap: 47464.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 41964.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_policy_engine(&flags::new_registry());
        let item = new_probe_policy("code-462", "label-670");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
