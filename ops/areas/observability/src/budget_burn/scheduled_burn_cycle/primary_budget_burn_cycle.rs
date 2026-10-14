use super::*;

use acme_kit::text;

/// Where a primary budget burn cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryBudgetBurnCycleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryBudgetBurnCycleStatus {
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

impl std::fmt::Display for PrimaryBudgetBurnCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary budget burn cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnCycle {
    pub code: String,
    pub label: String,
    pub host: String,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub channel: String,
    pub title: String,
    pub checksum: String,
    pub namespace: String,
    pub status: PrimaryBudgetBurnCycleStatus,
}

impl PrimaryBudgetBurnCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryBudgetBurnCycleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryBudgetBurnCycleStatus) -> bool {
        match self.status {
            PrimaryBudgetBurnCycleStatus::Open => {
                match next {
                    PrimaryBudgetBurnCycleStatus::InReview | PrimaryBudgetBurnCycleStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBudgetBurnCycleStatus::InReview => {
                return next == PrimaryBudgetBurnCycleStatus::Archived;
            }
            PrimaryBudgetBurnCycleStatus::Resolved => {
                return next == PrimaryBudgetBurnCycleStatus::Reopened
                    || next == PrimaryBudgetBurnCycleStatus::Archived;
            }
            PrimaryBudgetBurnCycleStatus::Reopened => {
                return next == PrimaryBudgetBurnCycleStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryBudgetBurnCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary budget burn cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> PrimaryBudgetBurnCycle {
        PrimaryBudgetBurnCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            channel: self.channel.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a primary budget burn cycle in status open.
pub fn new_primary_budget_burn_cycle(code: &str, label: &str) -> PrimaryBudgetBurnCycle {
    PrimaryBudgetBurnCycle {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        expires_at: 0,
        tags: Vec::new(),
        channel: "".to_string(),
        title: "".to_string(),
        checksum: "".to_string(),
        namespace: "".to_string(),
        status: PrimaryBudgetBurnCycleStatus::Open,
    }
}
