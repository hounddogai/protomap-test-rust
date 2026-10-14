use super::*;

use acme_kit::text;

/// Where a capped probe exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedProbeExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedProbeExceptionStatus {
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

impl std::fmt::Display for CappedProbeExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped probe exception as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeException {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub rotation_days: i64,
    pub limit: i64,
    pub issuer: String,
    pub contact_email: String,
    pub label: String,
    pub retries: i64,
    pub port: i64,
    pub status: CappedProbeExceptionStatus,
}

impl CappedProbeException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedProbeExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedProbeExceptionStatus) -> bool {
        match self.status {
            CappedProbeExceptionStatus::Idle => {
                match next {
                    CappedProbeExceptionStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedProbeExceptionStatus::Warming => {
                match next {
                    CappedProbeExceptionStatus::Ready | CappedProbeExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedProbeExceptionStatus::Ready => {
                return next == CappedProbeExceptionStatus::Stopped;
            }
            CappedProbeExceptionStatus::Draining => {
                return next == CappedProbeExceptionStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedProbeExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped probe exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.port as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedProbeException {
        CappedProbeException {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            rotation_days: self.rotation_days,
            limit: self.limit,
            issuer: self.issuer.clone(),
            contact_email: self.contact_email.clone(),
            label: self.label.clone(),
            retries: self.retries,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped probe exception in status idle.
pub fn new_capped_probe_exception(code: &str, title: &str) -> CappedProbeException {
    CappedProbeException {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        rotation_days: 0,
        limit: 0,
        issuer: "".to_string(),
        contact_email: "".to_string(),
        label: "".to_string(),
        retries: 0,
        port: 0,
        status: CappedProbeExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_probe_exception("code-3", "title-525");
        assert_eq!(item.status, CappedProbeExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_capped_probe_exception("code-3", "title-525");
        if item.move_to(CappedProbeExceptionStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedProbeExceptionStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_probe_exception("code-3", "title-525");
        item.status = CappedProbeExceptionStatus::Stopped;
        assert!(
            item.move_to(CappedProbeExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
