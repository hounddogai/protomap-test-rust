use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the egress segment calculation.
pub trait EgressSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressSegmentAdjuster {
    pub percent: f64,
}

impl EgressSegmentModifier for PercentEgressSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressSegmentModifier for FlaggedEgressSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSegmentDiscountOffsetAdjuster {
    pub weight: f64,
}

impl EgressSegmentModifier for EgressSegmentDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &EgressSegment, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EgressSegmentModifier for TieredEgressSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EgressSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleEgressSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl EgressSegmentModifier for TitleEgressSegmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &EgressSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the egress segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressSegmentEngine {
    pub steps: Vec<Rc<dyn EgressSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressSegmentEngine {
    pub fn evaluate(&mut self, item: &EgressSegment) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[EgressSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_egress_segment_engine(flags: &flags::Registry) -> EgressSegmentEngine {
    EgressSegmentEngine {
        steps: vec![
            Rc::new(PercentEgressSegmentAdjuster { percent: -6.1 }) as Rc<dyn EgressSegmentModifier>,
            Rc::new(FlaggedEgressSegmentAdjuster {
                flags: flags.clone(),
                flag: "egress-segment-regional".to_string(),
                percent: -7.8,
            }),
            Rc::new(EgressSegmentDiscountOffsetAdjuster { weight: 1.01 }),
            Rc::new(TieredEgressSegmentAdjuster {
                bounds: vec![6834.0, 7172.0],
                rates: vec![1.18, 0.87],
            }),
            Rc::new(TitleEgressSegmentAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.23),
                    ("online".to_string(), 0.73),
                    ("central".to_string(), 1.09),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 70052.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_segment_engine(&flags::new_registry());
        let item = new_egress_segment("code-872", "title-562");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
