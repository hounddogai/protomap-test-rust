use super::*;

use acme_kit::text;

/// Where a default egress reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultEgressReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultEgressReservationStatus {
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

impl std::fmt::Display for DefaultEgressReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default egress reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressReservation {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub contact_email: String,
    pub sequence: i64,
    pub cluster: String,
    pub maximum: f64,
    pub active: bool,
    pub channel: String,
    pub key_id: String,
    pub pinned: bool,
    pub status: DefaultEgressReservationStatus,
}

impl DefaultEgressReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultEgressReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultEgressReservationStatus) -> bool {
        match self.status {
            DefaultEgressReservationStatus::Idle => {
                match next {
                    DefaultEgressReservationStatus::Ready | DefaultEgressReservationStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEgressReservationStatus::Warming => {
                match next {
                    DefaultEgressReservationStatus::Ready | DefaultEgressReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultEgressReservationStatus::Ready => {
                return next == DefaultEgressReservationStatus::Draining
                    || next == DefaultEgressReservationStatus::Stopped;
            }
            DefaultEgressReservationStatus::Draining => {
                return next == DefaultEgressReservationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultEgressReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default egress reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        (self.rate * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultEgressReservation {
        DefaultEgressReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            contact_email: self.contact_email.clone(),
            sequence: self.sequence,
            cluster: self.cluster.clone(),
            maximum: self.maximum,
            active: self.active,
            channel: self.channel.clone(),
            key_id: self.key_id.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default egress reservation in status idle.
pub fn new_default_egress_reservation(code: &str, title: &str) -> DefaultEgressReservation {
    DefaultEgressReservation {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        contact_email: "".to_string(),
        sequence: 0,
        cluster: "".to_string(),
        maximum: 0.0,
        active: false,
        channel: "".to_string(),
        key_id: "".to_string(),
        pinned: false,
        status: DefaultEgressReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_egress_reservation("code-739", "title-355");
        assert_eq!(item.status, DefaultEgressReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_default_egress_reservation("code-739", "title-355");
        if item.move_to(DefaultEgressReservationStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultEgressReservationStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_egress_reservation("code-739", "title-355");
        item.status = DefaultEgressReservationStatus::Stopped;
        assert!(
            item.move_to(DefaultEgressReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
