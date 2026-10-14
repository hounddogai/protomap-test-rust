use super::*;

use acme_kit::text;

/// Where a reservation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReservationTemplateStatus {
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

impl std::fmt::Display for ReservationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One reservation template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationTemplate {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub rate: f64,
    pub rotation_days: i64,
    pub revision: i64,
    pub priority: i64,
    pub fingerprint: String,
    pub status: ReservationTemplateStatus,
}

impl ReservationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationTemplateStatus) -> bool {
        match self.status {
            ReservationTemplateStatus::Idle => {
                match next {
                    ReservationTemplateStatus::Ready | ReservationTemplateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationTemplateStatus::Warming => {
                return next == ReservationTemplateStatus::Stopped;
            }
            ReservationTemplateStatus::Ready => {
                return next == ReservationTemplateStatus::Draining;
            }
            ReservationTemplateStatus::Draining => {
                match next {
                    ReservationTemplateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ReservationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.expires_at as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationTemplate {
        ReservationTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            rate: self.rate,
            rotation_days: self.rotation_days,
            revision: self.revision,
            priority: self.priority,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a reservation template in status idle.
pub fn new_reservation_template(code: &str, title: &str) -> ReservationTemplate {
    ReservationTemplate {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        rate: 0.0,
        rotation_days: 0,
        revision: 0,
        priority: 0,
        fingerprint: "".to_string(),
        status: ReservationTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_reservation_template("code-816", "title-483");
        assert_eq!(item.status, ReservationTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_reservation_template("code-816", "title-483");
        if item.move_to(ReservationTemplateStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationTemplateStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_reservation_template("code-816", "title-483");
        item.status = ReservationTemplateStatus::Stopped;
        assert!(
            item.move_to(ReservationTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
