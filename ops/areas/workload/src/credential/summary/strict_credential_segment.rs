use super::*;

use acme_kit::text;

/// Where a strict credential segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictCredentialSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictCredentialSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for StrictCredentialSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict credential segment as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCredentialSegment {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub reviewer: String,
    pub notes: String,
    pub external_ref: String,
    pub country: String,
    pub status: StrictCredentialSegmentStatus,
}

impl StrictCredentialSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictCredentialSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictCredentialSegmentStatus) -> bool {
        match self.status {
            StrictCredentialSegmentStatus::New => {
                match next {
                    StrictCredentialSegmentStatus::Queued | StrictCredentialSegmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictCredentialSegmentStatus::Queued => {
                return next == StrictCredentialSegmentStatus::Done;
            }
            StrictCredentialSegmentStatus::Running => {
                match next {
                    StrictCredentialSegmentStatus::Done | StrictCredentialSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictCredentialSegmentStatus::Done => {
                match next {
                    StrictCredentialSegmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: StrictCredentialSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict credential segment {} cannot move from {} to {}",
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
        self.limit as f64
    }

    pub fn copy(&self) -> StrictCredentialSegment {
        StrictCredentialSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            reviewer: self.reviewer.clone(),
            notes: self.notes.clone(),
            external_ref: self.external_ref.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a strict credential segment in status new.
pub fn new_strict_credential_segment(code: &str, label: &str) -> StrictCredentialSegment {
    StrictCredentialSegment {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        reviewer: "".to_string(),
        notes: "".to_string(),
        external_ref: "".to_string(),
        country: "".to_string(),
        status: StrictCredentialSegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_strict_credential_segment("code-858", "label-158");
        assert_eq!(item.status, StrictCredentialSegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_strict_credential_segment("code-858", "label-158");
        if item.move_to(StrictCredentialSegmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictCredentialSegmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_strict_credential_segment("code-858", "label-158");
        item.status = StrictCredentialSegmentStatus::Failed;
        assert!(
            item.move_to(StrictCredentialSegmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
