use super::*;

use acme_kit::text;

/// Where a credential allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CredentialAllocationStatus {
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

impl std::fmt::Display for CredentialAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAllocation {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub updated_at: i64,
    pub port: i64,
    pub namespace: String,
    pub revision: i64,
    pub cipher: String,
    pub status: CredentialAllocationStatus,
}

impl CredentialAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialAllocationStatus) -> bool {
        match self.status {
            CredentialAllocationStatus::Planned => {
                match next {
                    CredentialAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialAllocationStatus::Scheduled => {
                return next == CredentialAllocationStatus::InProgress || next == CredentialAllocationStatus::Cancelled;
            }
            CredentialAllocationStatus::InProgress => {
                return next == CredentialAllocationStatus::Completed || next == CredentialAllocationStatus::Cancelled;
            }
            CredentialAllocationStatus::Completed => {
                match next {
                    CredentialAllocationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CredentialAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.revision as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialAllocation {
        CredentialAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            updated_at: self.updated_at,
            port: self.port,
            namespace: self.namespace.clone(),
            revision: self.revision,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a credential allocation in status planned.
pub fn new_credential_allocation(code: &str, label: &str) -> CredentialAllocation {
    CredentialAllocation {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        updated_at: 0,
        port: 0,
        namespace: "".to_string(),
        revision: 0,
        cipher: "".to_string(),
        status: CredentialAllocationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_credential_allocation("code-663", "label-639");
        assert_eq!(item.status, CredentialAllocationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_credential_allocation("code-663", "label-639");
        if item.move_to(CredentialAllocationStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialAllocationStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_credential_allocation("code-663", "label-639");
        item.status = CredentialAllocationStatus::Cancelled;
        assert!(
            item.move_to(CredentialAllocationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
