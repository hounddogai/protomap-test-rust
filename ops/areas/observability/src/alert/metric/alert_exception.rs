use super::*;

use acme_kit::text;

/// Where a alert exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AlertExceptionStatus {
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

impl std::fmt::Display for AlertExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertException {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub rotation_days: i64,
    pub host: String,
    pub limit: i64,
    pub issuer: String,
    pub expires_at: i64,
    pub ends_at: i64,
    pub ceiling: f64,
    pub status: AlertExceptionStatus,
}

impl AlertException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertExceptionStatus) -> bool {
        match self.status {
            AlertExceptionStatus::Idle => {
                return next == AlertExceptionStatus::Ready || next == AlertExceptionStatus::Stopped;
            }
            AlertExceptionStatus::Warming => {
                match next {
                    AlertExceptionStatus::Ready | AlertExceptionStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertExceptionStatus::Ready => {
                match next {
                    AlertExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertExceptionStatus::Draining => {
                return next == AlertExceptionStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "alert exception", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.limit as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertException {
        AlertException {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            rotation_days: self.rotation_days,
            host: self.host.clone(),
            limit: self.limit,
            issuer: self.issuer.clone(),
            expires_at: self.expires_at,
            ends_at: self.ends_at,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a alert exception in status idle.
pub fn new_alert_exception(code: &str, label: &str) -> AlertException {
    AlertException {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        rotation_days: 0,
        host: "".to_string(),
        limit: 0,
        issuer: "".to_string(),
        expires_at: 0,
        ends_at: 0,
        ceiling: 0.0,
        status: AlertExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_alert_exception("code-879", "label-816");
        assert_eq!(item.status, AlertExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_alert_exception("code-879", "label-816");
        if item.move_to(AlertExceptionStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertExceptionStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_alert_exception("code-879", "label-816");
        item.status = AlertExceptionStatus::Stopped;
        assert!(
            item.move_to(AlertExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
