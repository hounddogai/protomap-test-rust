use super::*;

use acme_kit::text;

/// Where a primary evidence request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEvidenceRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryEvidenceRequestStatus {
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

impl std::fmt::Display for PrimaryEvidenceRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary evidence request as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceRequest {
    pub code: String,
    pub name: String,
    pub category: String,
    pub rate: f64,
    pub duration_minutes: i64,
    pub phone: String,
    pub sequence: i64,
    pub approved: bool,
    pub rotation_days: i64,
    pub status: PrimaryEvidenceRequestStatus,
}

impl PrimaryEvidenceRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEvidenceRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEvidenceRequestStatus) -> bool {
        match self.status {
            PrimaryEvidenceRequestStatus::New => {
                match next {
                    PrimaryEvidenceRequestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceRequestStatus::Queued => {
                match next {
                    PrimaryEvidenceRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceRequestStatus::Running => {
                return next == PrimaryEvidenceRequestStatus::Done || next == PrimaryEvidenceRequestStatus::Failed;
            }
            PrimaryEvidenceRequestStatus::Done => {
                match next {
                    PrimaryEvidenceRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimaryEvidenceRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary evidence request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rotation_days as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEvidenceRequest {
        PrimaryEvidenceRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            rate: self.rate,
            duration_minutes: self.duration_minutes,
            phone: self.phone.clone(),
            sequence: self.sequence,
            approved: self.approved,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a primary evidence request in status new.
pub fn new_primary_evidence_request(code: &str, name: &str) -> PrimaryEvidenceRequest {
    PrimaryEvidenceRequest {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        rate: 0.0,
        duration_minutes: 0,
        phone: "".to_string(),
        sequence: 0,
        approved: false,
        rotation_days: 0,
        status: PrimaryEvidenceRequestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_evidence_request("code-375", "name-724");
        assert_eq!(item.status, PrimaryEvidenceRequestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_primary_evidence_request("code-375", "name-724");
        if item.move_to(PrimaryEvidenceRequestStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEvidenceRequestStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_evidence_request("code-375", "name-724");
        item.status = PrimaryEvidenceRequestStatus::Failed;
        assert!(
            item.move_to(PrimaryEvidenceRequestStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
