use super::*;

use acme_kit::text;

/// Where a verifier milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl VerifierMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for VerifierMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier milestone as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMilestone {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub ends_at: i64,
    pub tags: Vec<String>,
    pub external_ref: String,
    pub status: VerifierMilestoneStatus,
}

impl VerifierMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierMilestoneStatus) -> bool {
        match self.status {
            VerifierMilestoneStatus::Draft => {
                match next {
                    VerifierMilestoneStatus::Pending | VerifierMilestoneStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierMilestoneStatus::Pending => {
                return next == VerifierMilestoneStatus::Active;
            }
            VerifierMilestoneStatus::Active => {
                match next {
                    VerifierMilestoneStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierMilestoneStatus::Suspended => {
                match next {
                    VerifierMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: VerifierMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.retries as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierMilestone {
        VerifierMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            ends_at: self.ends_at,
            tags: self.tags.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a verifier milestone in status draft.
pub fn new_verifier_milestone(code: &str, title: &str) -> VerifierMilestone {
    VerifierMilestone {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        ends_at: 0,
        tags: Vec::new(),
        external_ref: "".to_string(),
        status: VerifierMilestoneStatus::Draft,
    }
}
