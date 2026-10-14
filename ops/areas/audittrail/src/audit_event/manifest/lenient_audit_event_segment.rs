use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient audit event segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientAuditEventSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientAuditEventSegmentStatus {
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

impl std::fmt::Display for LenientAuditEventSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient audit event segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventSegment {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub channel: String,
    pub window_days: i64,
    pub quantity: i64,
    pub status: LenientAuditEventSegmentStatus,
}

impl LenientAuditEventSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientAuditEventSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientAuditEventSegmentStatus) -> bool {
        match self.status {
            LenientAuditEventSegmentStatus::Idle => {
                return next == LenientAuditEventSegmentStatus::Ready
                    || next == LenientAuditEventSegmentStatus::Stopped;
            }
            LenientAuditEventSegmentStatus::Warming => {
                match next {
                    LenientAuditEventSegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAuditEventSegmentStatus::Ready => {
                return next == LenientAuditEventSegmentStatus::Stopped;
            }
            LenientAuditEventSegmentStatus::Draining => {
                match next {
                    LenientAuditEventSegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientAuditEventSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient audit event segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientAuditEventSegment {
        LenientAuditEventSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            channel: self.channel.clone(),
            window_days: self.window_days,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient audit event segment in status idle.
pub fn new_lenient_audit_event_segment(code: &str, title: &str) -> LenientAuditEventSegment {
    LenientAuditEventSegment {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        channel: "".to_string(),
        window_days: 0,
        quantity: 0,
        status: LenientAuditEventSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_audit_event_segment("code-133", "title-632");
        assert_eq!(item.status, LenientAuditEventSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_lenient_audit_event_segment("code-133", "title-632");
        if item.move_to(LenientAuditEventSegmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientAuditEventSegmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_audit_event_segment("code-133", "title-632");
        item.status = LenientAuditEventSegmentStatus::Stopped;
        assert!(
            item.move_to(LenientAuditEventSegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
