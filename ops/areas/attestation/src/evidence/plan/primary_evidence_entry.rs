use super::*;

use acme_kit::text;

/// Where a primary evidence entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEvidenceEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryEvidenceEntryStatus {
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

impl std::fmt::Display for PrimaryEvidenceEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary evidence entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceEntry {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub retries: i64,
    pub burst: i64,
    pub rate: f64,
    pub status: PrimaryEvidenceEntryStatus,
}

impl PrimaryEvidenceEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEvidenceEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEvidenceEntryStatus) -> bool {
        match self.status {
            PrimaryEvidenceEntryStatus::New => {
                match next {
                    PrimaryEvidenceEntryStatus::Running | PrimaryEvidenceEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceEntryStatus::Queued => {
                match next {
                    PrimaryEvidenceEntryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceEntryStatus::Running => {
                match next {
                    PrimaryEvidenceEntryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEvidenceEntryStatus::Done => {
                return next == PrimaryEvidenceEntryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryEvidenceEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary evidence entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary evidence entry", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.retries as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryEvidenceEntry {
        PrimaryEvidenceEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            retries: self.retries,
            burst: self.burst,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary evidence entry in status new.
pub fn new_primary_evidence_entry(code: &str, label: &str) -> PrimaryEvidenceEntry {
    PrimaryEvidenceEntry {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        retries: 0,
        burst: 0,
        rate: 0.0,
        status: PrimaryEvidenceEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_evidence_entry("code-365", "label-748");
        assert_eq!(item.status, PrimaryEvidenceEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_primary_evidence_entry("code-365", "label-748");
        if item.move_to(PrimaryEvidenceEntryStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEvidenceEntryStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_evidence_entry("code-365", "label-748");
        item.status = PrimaryEvidenceEntryStatus::Failed;
        assert!(
            item.move_to(PrimaryEvidenceEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
