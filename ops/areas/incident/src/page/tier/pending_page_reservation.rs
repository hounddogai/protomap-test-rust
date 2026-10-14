use super::*;

use acme_kit::text;

/// Where a pending page reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPageReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingPageReservationStatus {
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

impl std::fmt::Display for PendingPageReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending page reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageReservation {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub tenant_id: String,
    pub quantity: i64,
    pub version: i64,
    pub region: String,
    pub ttl_seconds: i64,
    pub issuer: String,
    pub ceiling: f64,
    pub status: PendingPageReservationStatus,
}

impl PendingPageReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPageReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPageReservationStatus) -> bool {
        match self.status {
            PendingPageReservationStatus::Idle => {
                return next == PendingPageReservationStatus::Warming;
            }
            PendingPageReservationStatus::Warming => {
                return next == PendingPageReservationStatus::Ready || next == PendingPageReservationStatus::Draining;
            }
            PendingPageReservationStatus::Ready => {
                match next {
                    PendingPageReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPageReservationStatus::Draining => {
                return next == PendingPageReservationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingPageReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending page reservation {} cannot move from {} to {}",
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

    pub fn weighted_value(&self) -> f64 {
        ((self.attempts as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPageReservation {
        PendingPageReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            tenant_id: self.tenant_id.clone(),
            quantity: self.quantity,
            version: self.version,
            region: self.region.clone(),
            ttl_seconds: self.ttl_seconds,
            issuer: self.issuer.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a pending page reservation in status idle.
pub fn new_pending_page_reservation(code: &str, label: &str) -> PendingPageReservation {
    PendingPageReservation {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        tenant_id: "".to_string(),
        quantity: 0,
        version: 0,
        region: "".to_string(),
        ttl_seconds: 0,
        issuer: "".to_string(),
        ceiling: 0.0,
        status: PendingPageReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_page_reservation("code-136", "label-50");
        assert_eq!(item.status, PendingPageReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_pending_page_reservation("code-136", "label-50");
        if item.move_to(PendingPageReservationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPageReservationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_page_reservation("code-136", "label-50");
        item.status = PendingPageReservationStatus::Stopped;
        assert!(
            item.move_to(PendingPageReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
