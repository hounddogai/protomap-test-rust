use super::*;

use acme_kit::text;

/// Where a lenient evidence notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEvidenceNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientEvidenceNoticeStatus {
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

impl std::fmt::Display for LenientEvidenceNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient evidence notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEvidenceNotice {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub label: String,
    pub duration_minutes: i64,
    pub status: LenientEvidenceNoticeStatus,
}

impl LenientEvidenceNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEvidenceNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEvidenceNoticeStatus) -> bool {
        match self.status {
            LenientEvidenceNoticeStatus::Open => {
                match next {
                    LenientEvidenceNoticeStatus::Resolved | LenientEvidenceNoticeStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEvidenceNoticeStatus::InReview => {
                match next {
                    LenientEvidenceNoticeStatus::Resolved | LenientEvidenceNoticeStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEvidenceNoticeStatus::Resolved => {
                return next == LenientEvidenceNoticeStatus::Archived;
            }
            LenientEvidenceNoticeStatus::Reopened => {
                return next == LenientEvidenceNoticeStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientEvidenceNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient evidence notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.sequence as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientEvidenceNotice {
        LenientEvidenceNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient evidence notice in status open.
pub fn new_lenient_evidence_notice(code: &str, title: &str) -> LenientEvidenceNotice {
    LenientEvidenceNotice {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        label: "".to_string(),
        duration_minutes: 0,
        status: LenientEvidenceNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_evidence_notice("code-345", "title-926");
        assert_eq!(item.status, LenientEvidenceNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_lenient_evidence_notice("code-345", "title-926");
        if item.move_to(LenientEvidenceNoticeStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientEvidenceNoticeStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_evidence_notice("code-345", "title-926");
        item.status = LenientEvidenceNoticeStatus::Archived;
        assert!(
            item.move_to(LenientEvidenceNoticeStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
