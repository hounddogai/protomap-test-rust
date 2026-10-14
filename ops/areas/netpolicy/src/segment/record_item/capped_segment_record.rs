use super::*;

use acme_kit::text;

/// Where a capped segment record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSegmentRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedSegmentRecordStatus {
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

impl std::fmt::Display for CappedSegmentRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped segment record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentRecord {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub phone: String,
    pub cipher: String,
    pub host: String,
    pub audience: String,
    pub timezone: String,
    pub name: String,
    pub archived: bool,
    pub margin: f64,
    pub status: CappedSegmentRecordStatus,
}

impl CappedSegmentRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSegmentRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSegmentRecordStatus) -> bool {
        match self.status {
            CappedSegmentRecordStatus::Open => {
                match next {
                    CappedSegmentRecordStatus::Resolved | CappedSegmentRecordStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSegmentRecordStatus::InReview => {
                return next == CappedSegmentRecordStatus::Reopened || next == CappedSegmentRecordStatus::Archived;
            }
            CappedSegmentRecordStatus::Resolved => {
                return next == CappedSegmentRecordStatus::Reopened || next == CappedSegmentRecordStatus::Archived;
            }
            CappedSegmentRecordStatus::Reopened => {
                return next == CappedSegmentRecordStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSegmentRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped segment record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped segment record", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.version as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSegmentRecord {
        CappedSegmentRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            phone: self.phone.clone(),
            cipher: self.cipher.clone(),
            host: self.host.clone(),
            audience: self.audience.clone(),
            timezone: self.timezone.clone(),
            name: self.name.clone(),
            archived: self.archived,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped segment record in status open.
pub fn new_capped_segment_record(code: &str, label: &str) -> CappedSegmentRecord {
    CappedSegmentRecord {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        phone: "".to_string(),
        cipher: "".to_string(),
        host: "".to_string(),
        audience: "".to_string(),
        timezone: "".to_string(),
        name: "".to_string(),
        archived: false,
        margin: 0.0,
        status: CappedSegmentRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_segment_record("code-820", "label-922");
        assert_eq!(item.status, CappedSegmentRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_capped_segment_record("code-820", "label-922");
        if item.move_to(CappedSegmentRecordStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedSegmentRecordStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_segment_record("code-820", "label-922");
        item.status = CappedSegmentRecordStatus::Archived;
        assert!(
            item.move_to(CappedSegmentRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
