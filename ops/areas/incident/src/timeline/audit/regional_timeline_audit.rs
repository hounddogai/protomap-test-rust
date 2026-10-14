use super::*;

use acme_kit::text;

/// Where a regional timeline audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTimelineAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalTimelineAuditStatus {
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

impl std::fmt::Display for RegionalTimelineAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional timeline audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineAudit {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub duration_minutes: i64,
    pub fingerprint: String,
    pub audience: String,
    pub status: RegionalTimelineAuditStatus,
}

impl RegionalTimelineAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTimelineAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTimelineAuditStatus) -> bool {
        match self.status {
            RegionalTimelineAuditStatus::Idle => {
                return next == RegionalTimelineAuditStatus::Warming || next == RegionalTimelineAuditStatus::Stopped;
            }
            RegionalTimelineAuditStatus::Warming => {
                match next {
                    RegionalTimelineAuditStatus::Ready | RegionalTimelineAuditStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTimelineAuditStatus::Ready => {
                return next == RegionalTimelineAuditStatus::Stopped;
            }
            RegionalTimelineAuditStatus::Draining => {
                match next {
                    RegionalTimelineAuditStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalTimelineAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional timeline audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional timeline audit", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.duration_minutes as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTimelineAudit {
        RegionalTimelineAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            duration_minutes: self.duration_minutes,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional timeline audit in status idle.
pub fn new_regional_timeline_audit(code: &str, title: &str) -> RegionalTimelineAudit {
    RegionalTimelineAudit {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        duration_minutes: 0,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        status: RegionalTimelineAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_timeline_audit("code-816", "title-470");
        assert_eq!(item.status, RegionalTimelineAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_regional_timeline_audit("code-816", "title-470");
        if item.move_to(RegionalTimelineAuditStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalTimelineAuditStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_timeline_audit("code-816", "title-470");
        item.status = RegionalTimelineAuditStatus::Stopped;
        assert!(
            item.move_to(RegionalTimelineAuditStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
