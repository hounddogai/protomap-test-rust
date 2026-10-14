use super::*;

use acme_kit::text;

/// Where a primary sink draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySinkDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimarySinkDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimarySinkDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary sink draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkDraft {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub notes: String,
    pub cluster: String,
    pub quantity: i64,
    pub reviewer: String,
    pub ratio: f64,
    pub limit: i64,
    pub audience: String,
    pub status: PrimarySinkDraftStatus,
}

impl PrimarySinkDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySinkDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySinkDraftStatus) -> bool {
        match self.status {
            PrimarySinkDraftStatus::Proposed => {
                match next {
                    PrimarySinkDraftStatus::Rejected | PrimarySinkDraftStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySinkDraftStatus::Approved => {
                match next {
                    PrimarySinkDraftStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySinkDraftStatus::Rejected => {
                return next == PrimarySinkDraftStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimarySinkDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary sink draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary sink draft", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.limit as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySinkDraft {
        PrimarySinkDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            notes: self.notes.clone(),
            cluster: self.cluster.clone(),
            quantity: self.quantity,
            reviewer: self.reviewer.clone(),
            ratio: self.ratio,
            limit: self.limit,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary sink draft in status proposed.
pub fn new_primary_sink_draft(code: &str, name: &str) -> PrimarySinkDraft {
    PrimarySinkDraft {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        notes: "".to_string(),
        cluster: "".to_string(),
        quantity: 0,
        reviewer: "".to_string(),
        ratio: 0.0,
        limit: 0,
        audience: "".to_string(),
        status: PrimarySinkDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_sink_draft("code-393", "name-511");
        assert_eq!(item.status, PrimarySinkDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_primary_sink_draft("code-393", "name-511");
        if item.move_to(PrimarySinkDraftStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySinkDraftStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_sink_draft("code-393", "name-511");
        item.status = PrimarySinkDraftStatus::Withdrawn;
        assert!(
            item.move_to(PrimarySinkDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
