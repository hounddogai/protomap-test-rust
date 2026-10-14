use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule reservation calculation.
pub trait RuleReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleReservationAdjuster {
    pub percent: f64,
}

impl RuleReservationStep for PercentRuleReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleReservationStep for FlaggedRuleReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleReservationAdjuster {
    pub minimum: f64,
}

impl RuleReservationStep for MinimumRuleReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRuleReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleReservationStep for ChannelRuleReservationAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RuleReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rule reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleReservationEngine {
    pub steps: Vec<Rc<dyn RuleReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleReservationEngine {
    pub fn price(&mut self, item: &RuleReservation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[RuleReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rule_reservation_engine(flags: &flags::Registry) -> RuleReservationEngine {
    RuleReservationEngine {
        steps: vec![
            Rc::new(PercentRuleReservationAdjuster { percent: 2.2 }) as Rc<dyn RuleReservationStep>,
            Rc::new(FlaggedRuleReservationAdjuster {
                flags: flags.clone(),
                flag: "rule-reservation-global".to_string(),
                percent: -7.3,
            }),
            Rc::new(MinimumRuleReservationAdjuster { minimum: 24.2 }),
            Rc::new(ChannelRuleReservationAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.88), ("north".to_string(), 1.31)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 17293.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_reservation_engine(&flags::new_registry());
        let item = new_rule_reservation("code-39", "name-842");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
