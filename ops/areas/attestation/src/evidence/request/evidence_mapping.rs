use super::*;

use acme_kit::text;

/// Where a evidence mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EvidenceMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for EvidenceMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceMapping {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub locale: String,
    pub cluster: String,
    pub status: EvidenceMappingStatus,
}

impl EvidenceMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceMappingStatus) -> bool {
        match self.status {
            EvidenceMappingStatus::Planned => {
                return next == EvidenceMappingStatus::InProgress || next == EvidenceMappingStatus::Cancelled;
            }
            EvidenceMappingStatus::Scheduled => {
                return next == EvidenceMappingStatus::Cancelled;
            }
            EvidenceMappingStatus::InProgress => {
                return next == EvidenceMappingStatus::Completed;
            }
            EvidenceMappingStatus::Completed => {
                match next {
                    EvidenceMappingStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EvidenceMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> EvidenceMapping {
        EvidenceMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            locale: self.locale.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a evidence mapping in status planned.
pub fn new_evidence_mapping(code: &str, label: &str) -> EvidenceMapping {
    EvidenceMapping {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        locale: "".to_string(),
        cluster: "".to_string(),
        status: EvidenceMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_evidence_mapping("code-659", "label-73");
        assert_eq!(item.status, EvidenceMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_evidence_mapping("code-659", "label-73");
        if item.move_to(EvidenceMappingStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceMappingStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_evidence_mapping("code-659", "label-73");
        item.status = EvidenceMappingStatus::Cancelled;
        assert!(
            item.move_to(EvidenceMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
