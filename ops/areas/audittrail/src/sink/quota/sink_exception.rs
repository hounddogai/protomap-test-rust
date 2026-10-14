use super::*;

use acme_kit::text;

/// Where a sink exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SinkExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SinkExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkException {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub grace_days: i64,
    pub burst: i64,
    pub starts_at: i64,
    pub contact_email: String,
    pub active: bool,
    pub namespace: String,
    pub status: SinkExceptionStatus,
}

impl SinkException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkExceptionStatus) -> bool {
        match self.status {
            SinkExceptionStatus::Proposed => {
                match next {
                    SinkExceptionStatus::Approved | SinkExceptionStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkExceptionStatus::Approved => {
                return next == SinkExceptionStatus::Rejected || next == SinkExceptionStatus::Withdrawn;
            }
            SinkExceptionStatus::Rejected => {
                return next == SinkExceptionStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink exception", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.starts_at as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkException {
        SinkException {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            grace_days: self.grace_days,
            burst: self.burst,
            starts_at: self.starts_at,
            contact_email: self.contact_email.clone(),
            active: self.active,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a sink exception in status proposed.
pub fn new_sink_exception(code: &str, name: &str) -> SinkException {
    SinkException {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        grace_days: 0,
        burst: 0,
        starts_at: 0,
        contact_email: "".to_string(),
        active: false,
        namespace: "".to_string(),
        status: SinkExceptionStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_sink_exception("code-25", "name-942");
        assert_eq!(item.status, SinkExceptionStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_sink_exception("code-25", "name-942");
        if item.move_to(SinkExceptionStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkExceptionStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_sink_exception("code-25", "name-942");
        item.status = SinkExceptionStatus::Withdrawn;
        assert!(
            item.move_to(SinkExceptionStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
