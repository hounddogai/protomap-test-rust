use super::*;

use acme_kit::text;

/// Where a default segment notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSegmentNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultSegmentNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for DefaultSegmentNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default segment notice as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSegmentNotice {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub score: f64,
    pub principal: String,
    pub duration_minutes: i64,
    pub cipher: String,
    pub batch_size: i64,
    pub pinned: bool,
    pub reviewer: String,
    pub active: bool,
    pub status: DefaultSegmentNoticeStatus,
}

impl DefaultSegmentNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSegmentNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSegmentNoticeStatus) -> bool {
        match self.status {
            DefaultSegmentNoticeStatus::Draft => {
                match next {
                    DefaultSegmentNoticeStatus::Pending | DefaultSegmentNoticeStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSegmentNoticeStatus::Pending => {
                match next {
                    DefaultSegmentNoticeStatus::Active | DefaultSegmentNoticeStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSegmentNoticeStatus::Active => {
                match next {
                    DefaultSegmentNoticeStatus::Suspended | DefaultSegmentNoticeStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSegmentNoticeStatus::Suspended => {
                return next == DefaultSegmentNoticeStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultSegmentNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default segment notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultSegmentNotice {
        DefaultSegmentNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            score: self.score,
            principal: self.principal.clone(),
            duration_minutes: self.duration_minutes,
            cipher: self.cipher.clone(),
            batch_size: self.batch_size,
            pinned: self.pinned,
            reviewer: self.reviewer.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default segment notice in status draft.
pub fn new_default_segment_notice(code: &str, label: &str) -> DefaultSegmentNotice {
    DefaultSegmentNotice {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        score: 0.0,
        principal: "".to_string(),
        duration_minutes: 0,
        cipher: "".to_string(),
        batch_size: 0,
        pinned: false,
        reviewer: "".to_string(),
        active: false,
        status: DefaultSegmentNoticeStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_segment_notice("code-721", "label-734");
        assert_eq!(item.status, DefaultSegmentNoticeStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_default_segment_notice("code-721", "label-734");
        if item.move_to(DefaultSegmentNoticeStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultSegmentNoticeStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_segment_notice("code-721", "label-734");
        item.status = DefaultSegmentNoticeStatus::Closed;
        assert!(
            item.move_to(DefaultSegmentNoticeStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
