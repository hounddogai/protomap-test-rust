use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a allowlist proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AllowlistProposalStatus {
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

impl std::fmt::Display for AllowlistProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One allowlist proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistProposal {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub batch_size: i64,
    pub timezone: String,
    pub floor_value: f64,
    pub created_at: i64,
    pub grace_days: i64,
    pub updated_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub status: AllowlistProposalStatus,
}

impl AllowlistProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistProposalStatus) -> bool {
        match self.status {
            AllowlistProposalStatus::New => {
                return next == AllowlistProposalStatus::Queued || next == AllowlistProposalStatus::Done;
            }
            AllowlistProposalStatus::Queued => {
                match next {
                    AllowlistProposalStatus::Running | AllowlistProposalStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistProposalStatus::Running => {
                match next {
                    AllowlistProposalStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistProposalStatus::Done => {
                return next == AllowlistProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.floor_value - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistProposal {
        AllowlistProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            batch_size: self.batch_size,
            timezone: self.timezone.clone(),
            floor_value: self.floor_value,
            created_at: self.created_at,
            grace_days: self.grace_days,
            updated_at: self.updated_at,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a allowlist proposal in status new.
pub fn new_allowlist_proposal(code: &str, title: &str) -> AllowlistProposal {
    AllowlistProposal {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        batch_size: 0,
        timezone: "".to_string(),
        floor_value: 0.0,
        created_at: 0,
        grace_days: 0,
        updated_at: 0,
        attributes: BTreeMap::new(),
        status: AllowlistProposalStatus::New,
    }
}
