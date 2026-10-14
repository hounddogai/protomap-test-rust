use super::*;

use acme_kit::text;

/// Where a regional audit event snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalAuditEventSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalAuditEventSnapshotStatus {
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

impl std::fmt::Display for RegionalAuditEventSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional audit event snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAuditEventSnapshot {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub revision: i64,
    pub rate: f64,
    pub visible: bool,
    pub ceiling: f64,
    pub status: RegionalAuditEventSnapshotStatus,
}

impl RegionalAuditEventSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalAuditEventSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalAuditEventSnapshotStatus) -> bool {
        match self.status {
            RegionalAuditEventSnapshotStatus::Idle => {
                match next {
                    RegionalAuditEventSnapshotStatus::Ready | RegionalAuditEventSnapshotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalAuditEventSnapshotStatus::Warming => {
                return next == RegionalAuditEventSnapshotStatus::Ready
                    || next == RegionalAuditEventSnapshotStatus::Draining;
            }
            RegionalAuditEventSnapshotStatus::Ready => {
                return next == RegionalAuditEventSnapshotStatus::Draining
                    || next == RegionalAuditEventSnapshotStatus::Stopped;
            }
            RegionalAuditEventSnapshotStatus::Draining => {
                return next == RegionalAuditEventSnapshotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalAuditEventSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional audit event snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.revision as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalAuditEventSnapshot {
        RegionalAuditEventSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            revision: self.revision,
            rate: self.rate,
            visible: self.visible,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional audit event snapshot in status idle.
pub fn new_regional_audit_event_snapshot(code: &str, title: &str) -> RegionalAuditEventSnapshot {
    RegionalAuditEventSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        revision: 0,
        rate: 0.0,
        visible: false,
        ceiling: 0.0,
        status: RegionalAuditEventSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_audit_event_snapshot("code-776", "title-134");
        assert_eq!(item.status, RegionalAuditEventSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_regional_audit_event_snapshot("code-776", "title-134");
        if item.move_to(RegionalAuditEventSnapshotStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalAuditEventSnapshotStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_audit_event_snapshot("code-776", "title-134");
        item.status = RegionalAuditEventSnapshotStatus::Stopped;
        assert!(
            item.move_to(RegionalAuditEventSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
