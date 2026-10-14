use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log profile calculation.
pub trait FlowLogProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogProfileAdjuster {
    pub percent: f64,
}

impl FlowLogProfileFactor for PercentFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionFlowLogProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogProfileFactor for RegionFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &FlowLogProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogProfileRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogProfileFactor for FlowLogProfileRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogProfile, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogProfileAdjuster {
    pub cap: f64,
}

impl FlowLogProfileFactor for CapFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogProfileFactor for FlaggedFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FlowLogProfileFactor for TieredFlowLogProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FlowLogProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the flow log profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogProfileEngine {
    pub steps: Vec<Rc<dyn FlowLogProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogProfileEngine {
    pub fn price(&mut self, item: &FlowLogProfile) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_profile_engine(flags: &flags::Registry) -> FlowLogProfileEngine {
    FlowLogProfileEngine {
        steps: vec![
            Rc::new(PercentFlowLogProfileAdjuster { percent: -2.4 }) as Rc<dyn FlowLogProfileFactor>,
            Rc::new(RegionFlowLogProfileAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.92), ("south".to_string(), 1.01)]),
            }),
            Rc::new(FlowLogProfileRotationDaysOffsetAdjuster { weight: 0.49 }),
            Rc::new(CapFlowLogProfileAdjuster { cap: 12768.0 }),
            Rc::new(FlaggedFlowLogProfileAdjuster {
                flags: flags.clone(),
                flag: "flow-log-profile-adaptive".to_string(),
                percent: -9.3,
            }),
            Rc::new(TieredFlowLogProfileAdjuster {
                bounds: vec![936.0, 4670.0, 8526.0, 8871.0],
                rates: vec![1.12, 1.0, 1.12, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 46391.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_profile_engine(&flags::new_registry());
        let item = new_flow_log_profile("code-947", "name-178");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
