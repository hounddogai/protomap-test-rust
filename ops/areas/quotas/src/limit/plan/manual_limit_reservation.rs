use super::*;

use acme_kit::text;

/// Where a manual limit reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualLimitReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualLimitReservationStatus {
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

impl std::fmt::Display for ManualLimitReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual limit reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitReservation {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub priority: i64,
    pub capacity: i64,
    pub burst: i64,
    pub namespace: String,
    pub tenant_id: String,
    pub approved: bool,
    pub issuer: String,
    pub version: i64,
    pub status: ManualLimitReservationStatus,
}

impl ManualLimitReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualLimitReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualLimitReservationStatus) -> bool {
        match self.status {
            ManualLimitReservationStatus::Idle => {
                match next {
                    ManualLimitReservationStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualLimitReservationStatus::Warming => {
                match next {
                    ManualLimitReservationStatus::Ready | ManualLimitReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualLimitReservationStatus::Ready => {
                return next == ManualLimitReservationStatus::Stopped;
            }
            ManualLimitReservationStatus::Draining => {
                match next {
                    ManualLimitReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualLimitReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual limit reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.capacity as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualLimitReservation {
        ManualLimitReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            priority: self.priority,
            capacity: self.capacity,
            burst: self.burst,
            namespace: self.namespace.clone(),
            tenant_id: self.tenant_id.clone(),
            approved: self.approved,
            issuer: self.issuer.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual limit reservation in status idle.
pub fn new_manual_limit_reservation(code: &str, label: &str) -> ManualLimitReservation {
    ManualLimitReservation {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        priority: 0,
        capacity: 0,
        burst: 0,
        namespace: "".to_string(),
        tenant_id: "".to_string(),
        approved: false,
        issuer: "".to_string(),
        version: 0,
        status: ManualLimitReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_limit_reservation("code-636", "label-284");
        assert_eq!(item.status, ManualLimitReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_manual_limit_reservation("code-636", "label-284");
        if item.move_to(ManualLimitReservationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualLimitReservationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_limit_reservation("code-636", "label-284");
        item.status = ManualLimitReservationStatus::Stopped;
        assert!(
            item.move_to(ManualLimitReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
