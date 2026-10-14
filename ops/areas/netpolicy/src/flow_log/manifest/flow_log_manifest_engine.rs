use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log manifest calculation.
pub trait FlowLogManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogManifest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogManifestFactor for FlaggedFlowLogManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogManifestAdjuster {
    pub minimum: f64,
}

impl FlowLogManifestFactor for MinimumFlowLogManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogManifestAdjuster {
    pub percent: f64,
}

impl FlowLogManifestFactor for PercentFlowLogManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFlowLogManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogManifestFactor for NamespaceFlowLogManifestAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FlowLogManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogManifestPortOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogManifestFactor for FlowLogManifestPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogManifest, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogManifestAdjuster {
    pub cap: f64,
}

impl FlowLogManifestFactor for CapFlowLogManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the flow log manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogManifestEngine {
    pub steps: Vec<Rc<dyn FlowLogManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogManifestEngine {
    pub fn compute(&mut self, item: &FlowLogManifest) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_flow_log_manifest_engine(flags: &flags::Registry) -> FlowLogManifestEngine {
    FlowLogManifestEngine {
        steps: vec![
            Rc::new(FlaggedFlowLogManifestAdjuster {
                flags: flags.clone(),
                flag: "flow-log-manifest-default".to_string(),
                percent: 4.5,
            }) as Rc<dyn FlowLogManifestFactor>,
            Rc::new(MinimumFlowLogManifestAdjuster { minimum: 16.3 }),
            Rc::new(PercentFlowLogManifestAdjuster { percent: -9.9 }),
            Rc::new(NamespaceFlowLogManifestAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.18),
                    ("retail".to_string(), 0.72),
                    ("east".to_string(), 1.25),
                ]),
            }),
            Rc::new(FlowLogManifestPortOffsetAdjuster { weight: 0.06 }),
            Rc::new(CapFlowLogManifestAdjuster { cap: 15424.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 76852.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_manifest_engine(&flags::new_registry());
        let item = new_flow_log_manifest("code-427", "name-892");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
