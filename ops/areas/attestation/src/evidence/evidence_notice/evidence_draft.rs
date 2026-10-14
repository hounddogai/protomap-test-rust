use super::*;

use acme_kit::text;

/// Where a evidence draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EvidenceDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EvidenceDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceDraft {
    pub code: String,
    pub label: String,
    pub category: String,
    pub floor_value: f64,
    pub attempts: i64,
    pub margin: f64,
    pub checksum: String,
    pub revision: i64,
    pub limit: i64,
    pub port: i64,
    pub status: EvidenceDraftStatus,
}

impl EvidenceDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceDraftStatus) -> bool {
        match self.status {
            EvidenceDraftStatus::Created => {
                match next {
                    EvidenceDraftStatus::Validated | EvidenceDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceDraftStatus::Validated => {
                match next {
                    EvidenceDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceDraftStatus::Published => {
                return next == EvidenceDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "evidence draft", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.port as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceDraft {
        EvidenceDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            floor_value: self.floor_value,
            attempts: self.attempts,
            margin: self.margin,
            checksum: self.checksum.clone(),
            revision: self.revision,
            limit: self.limit,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a evidence draft in status created.
pub fn new_evidence_draft(code: &str, label: &str) -> EvidenceDraft {
    EvidenceDraft {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        floor_value: 0.0,
        attempts: 0,
        margin: 0.0,
        checksum: "".to_string(),
        revision: 0,
        limit: 0,
        port: 0,
        status: EvidenceDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_evidence_draft("code-288", "label-865");
        assert_eq!(item.status, EvidenceDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_evidence_draft("code-288", "label-865");
        if item.move_to(EvidenceDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_evidence_draft("code-288", "label-865");
        item.status = EvidenceDraftStatus::Retired;
        assert!(
            item.move_to(EvidenceDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
