use super::*;

use acme_kit::text;

/// Where a alert digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AlertDigestStatus {
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

impl std::fmt::Display for AlertDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertDigest {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub rate: f64,
    pub currency: String,
    pub created_at: i64,
    pub weight: f64,
    pub visible: bool,
    pub principal: String,
    pub sequence: i64,
    pub notes: String,
    pub status: AlertDigestStatus,
}

impl AlertDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertDigestStatus) -> bool {
        match self.status {
            AlertDigestStatus::Idle => {
                return next == AlertDigestStatus::Warming || next == AlertDigestStatus::Draining;
            }
            AlertDigestStatus::Warming => {
                match next {
                    AlertDigestStatus::Draining | AlertDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertDigestStatus::Ready => {
                match next {
                    AlertDigestStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertDigestStatus::Draining => {
                match next {
                    AlertDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AlertDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.rate * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertDigest {
        AlertDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            rate: self.rate,
            currency: self.currency.clone(),
            created_at: self.created_at,
            weight: self.weight,
            visible: self.visible,
            principal: self.principal.clone(),
            sequence: self.sequence,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a alert digest in status idle.
pub fn new_alert_digest(code: &str, label: &str) -> AlertDigest {
    AlertDigest {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        rate: 0.0,
        currency: "".to_string(),
        created_at: 0,
        weight: 0.0,
        visible: false,
        principal: "".to_string(),
        sequence: 0,
        notes: "".to_string(),
        status: AlertDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_alert_digest("code-350", "label-712");
        assert_eq!(item.status, AlertDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_alert_digest("code-350", "label-712");
        if item.move_to(AlertDigestStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertDigestStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_alert_digest("code-350", "label-712");
        item.status = AlertDigestStatus::Stopped;
        assert!(item.move_to(AlertDigestStatus::Idle).is_err(), "expected move to fail");
    }
}
