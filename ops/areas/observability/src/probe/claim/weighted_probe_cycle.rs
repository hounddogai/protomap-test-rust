use super::*;

use acme_kit::text;

/// Where a weighted probe cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedProbeCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedProbeCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedProbeCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted probe cycle as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedProbeCycle {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub amount: f64,
    pub reviewer: String,
    pub unit: String,
    pub discount: f64,
    pub description: String,
    pub region: String,
    pub score: f64,
    pub phone: String,
    pub status: WeightedProbeCycleStatus,
}

impl WeightedProbeCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedProbeCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedProbeCycleStatus) -> bool {
        match self.status {
            WeightedProbeCycleStatus::Created => {
                match next {
                    WeightedProbeCycleStatus::Validated | WeightedProbeCycleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedProbeCycleStatus::Validated => {
                return next == WeightedProbeCycleStatus::Published || next == WeightedProbeCycleStatus::Retired;
            }
            WeightedProbeCycleStatus::Published => {
                return next == WeightedProbeCycleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedProbeCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted probe cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.amount + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedProbeCycle {
        WeightedProbeCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            amount: self.amount,
            reviewer: self.reviewer.clone(),
            unit: self.unit.clone(),
            discount: self.discount,
            description: self.description.clone(),
            region: self.region.clone(),
            score: self.score,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a weighted probe cycle in status created.
pub fn new_weighted_probe_cycle(code: &str, label: &str) -> WeightedProbeCycle {
    WeightedProbeCycle {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        amount: 0.0,
        reviewer: "".to_string(),
        unit: "".to_string(),
        discount: 0.0,
        description: "".to_string(),
        region: "".to_string(),
        score: 0.0,
        phone: "".to_string(),
        status: WeightedProbeCycleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_probe_cycle("code-293", "label-386");
        assert_eq!(item.status, WeightedProbeCycleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_weighted_probe_cycle("code-293", "label-386");
        if item.move_to(WeightedProbeCycleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedProbeCycleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_probe_cycle("code-293", "label-386");
        item.status = WeightedProbeCycleStatus::Retired;
        assert!(
            item.move_to(WeightedProbeCycleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
