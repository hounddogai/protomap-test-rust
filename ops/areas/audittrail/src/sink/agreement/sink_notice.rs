use super::*;

use acme_kit::text;

/// Where a sink notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SinkNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SinkNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkNotice {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub owner_id: String,
    pub discount: f64,
    pub status: SinkNoticeStatus,
}

impl SinkNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkNoticeStatus) -> bool {
        match self.status {
            SinkNoticeStatus::Open => {
                match next {
                    SinkNoticeStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkNoticeStatus::InReview => {
                match next {
                    SinkNoticeStatus::Resolved | SinkNoticeStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkNoticeStatus::Resolved => {
                match next {
                    SinkNoticeStatus::Reopened | SinkNoticeStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkNoticeStatus::Reopened => {
                return next == SinkNoticeStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> SinkNotice {
        SinkNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            owner_id: self.owner_id.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a sink notice in status open.
pub fn new_sink_notice(code: &str, label: &str) -> SinkNotice {
    SinkNotice {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        owner_id: "".to_string(),
        discount: 0.0,
        status: SinkNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_sink_notice("code-673", "label-896");
        assert_eq!(item.status, SinkNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_sink_notice("code-673", "label-896");
        if item.move_to(SinkNoticeStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkNoticeStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_sink_notice("code-673", "label-896");
        item.status = SinkNoticeStatus::Archived;
        assert!(item.move_to(SinkNoticeStatus::Open).is_err(), "expected move to fail");
    }
}
