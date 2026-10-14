use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a reservation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ReservationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation budget as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationBudget {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub attributes: BTreeMap<String, String>,
    pub pinned: bool,
    pub visible: bool,
    pub maximum: f64,
    pub priority: i64,
    pub port: i64,
    pub status: ReservationBudgetStatus,
}

impl ReservationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationBudgetStatus) -> bool {
        match self.status {
            ReservationBudgetStatus::Idle => {
                match next {
                    ReservationBudgetStatus::Warming | ReservationBudgetStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationBudgetStatus::Warming => {
                match next {
                    ReservationBudgetStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationBudgetStatus::Ready => {
                return next == ReservationBudgetStatus::Stopped;
            }
            ReservationBudgetStatus::Draining => {
                return next == ReservationBudgetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "reservation budget", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.retries as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationBudget {
        ReservationBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            attributes: self.attributes.clone(),
            pinned: self.pinned,
            visible: self.visible,
            maximum: self.maximum,
            priority: self.priority,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a reservation budget in status idle.
pub fn new_reservation_budget(code: &str, title: &str) -> ReservationBudget {
    ReservationBudget {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        attributes: BTreeMap::new(),
        pinned: false,
        visible: false,
        maximum: 0.0,
        priority: 0,
        port: 0,
        status: ReservationBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_reservation_budget("code-985", "title-807");
        assert_eq!(item.status, ReservationBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_reservation_budget("code-985", "title-807");
        if item.move_to(ReservationBudgetStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationBudgetStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_reservation_budget("code-985", "title-807");
        item.status = ReservationBudgetStatus::Stopped;
        assert!(
            item.move_to(ReservationBudgetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
