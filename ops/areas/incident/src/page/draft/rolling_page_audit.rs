use super::*;

use acme_kit::text;

/// Where a rolling page audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPageAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingPageAuditStatus {
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

impl std::fmt::Display for RollingPageAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling page audit as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageAudit {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub fingerprint: String,
    pub scope: String,
    pub minimum: f64,
    pub starts_at: i64,
    pub priority: i64,
    pub floor_value: f64,
    pub segment: String,
    pub burst: i64,
    pub status: RollingPageAuditStatus,
}

impl RollingPageAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPageAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPageAuditStatus) -> bool {
        match self.status {
            RollingPageAuditStatus::Idle => {
                return next == RollingPageAuditStatus::Draining;
            }
            RollingPageAuditStatus::Warming => {
                match next {
                    RollingPageAuditStatus::Ready | RollingPageAuditStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPageAuditStatus::Ready => {
                return next == RollingPageAuditStatus::Stopped;
            }
            RollingPageAuditStatus::Draining => {
                match next {
                    RollingPageAuditStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RollingPageAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling page audit {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.burst as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPageAudit {
        RollingPageAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            fingerprint: self.fingerprint.clone(),
            scope: self.scope.clone(),
            minimum: self.minimum,
            starts_at: self.starts_at,
            priority: self.priority,
            floor_value: self.floor_value,
            segment: self.segment.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rolling page audit in status idle.
pub fn new_rolling_page_audit(code: &str, title: &str) -> RollingPageAudit {
    RollingPageAudit {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        fingerprint: "".to_string(),
        scope: "".to_string(),
        minimum: 0.0,
        starts_at: 0,
        priority: 0,
        floor_value: 0.0,
        segment: "".to_string(),
        burst: 0,
        status: RollingPageAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_page_audit("code-674", "title-720");
        assert_eq!(item.status, RollingPageAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_rolling_page_audit("code-674", "title-720");
        if item.move_to(RollingPageAuditStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPageAuditStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_page_audit("code-674", "title-720");
        item.status = RollingPageAuditStatus::Stopped;
        assert!(
            item.move_to(RollingPageAuditStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
