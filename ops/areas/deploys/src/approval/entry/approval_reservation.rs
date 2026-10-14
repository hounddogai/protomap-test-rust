use super::*;

use acme_kit::text;

/// Where a approval reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ApprovalReservationStatus {
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

impl std::fmt::Display for ApprovalReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalReservation {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub starts_at: i64,
    pub ttl_seconds: i64,
    pub expires_at: i64,
    pub audience: String,
    pub status: ApprovalReservationStatus,
}

impl ApprovalReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalReservationStatus) -> bool {
        match self.status {
            ApprovalReservationStatus::Idle => {
                match next {
                    ApprovalReservationStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalReservationStatus::Warming => {
                match next {
                    ApprovalReservationStatus::Ready | ApprovalReservationStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalReservationStatus::Ready => {
                match next {
                    ApprovalReservationStatus::Draining | ApprovalReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalReservationStatus::Draining => {
                match next {
                    ApprovalReservationStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ApprovalReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "approval reservation", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.starts_at as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalReservation {
        ApprovalReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            starts_at: self.starts_at,
            ttl_seconds: self.ttl_seconds,
            expires_at: self.expires_at,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a approval reservation in status idle.
pub fn new_approval_reservation(code: &str, title: &str) -> ApprovalReservation {
    ApprovalReservation {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        starts_at: 0,
        ttl_seconds: 0,
        expires_at: 0,
        audience: "".to_string(),
        status: ApprovalReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_approval_reservation("code-294", "title-577");
        assert_eq!(item.status, ApprovalReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_approval_reservation("code-294", "title-577");
        if item.move_to(ApprovalReservationStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalReservationStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_approval_reservation("code-294", "title-577");
        item.status = ApprovalReservationStatus::Stopped;
        assert!(
            item.move_to(ApprovalReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
