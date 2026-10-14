use super::*;

use acme_kit::text;

/// Where a evidence notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EvidenceNoticeStatus {
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

impl std::fmt::Display for EvidenceNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One evidence notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceNotice {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub active: bool,
    pub archived: bool,
    pub starts_at: i64,
    pub locale: String,
    pub limit: i64,
    pub budget: f64,
    pub created_at: i64,
    pub sequence: i64,
    pub status: EvidenceNoticeStatus,
}

impl EvidenceNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceNoticeStatus) -> bool {
        match self.status {
            EvidenceNoticeStatus::Planned => {
                match next {
                    EvidenceNoticeStatus::Completed | EvidenceNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceNoticeStatus::Scheduled => {
                match next {
                    EvidenceNoticeStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceNoticeStatus::InProgress => {
                match next {
                    EvidenceNoticeStatus::Completed | EvidenceNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceNoticeStatus::Completed => {
                match next {
                    EvidenceNoticeStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EvidenceNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.budget + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceNotice {
        EvidenceNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            active: self.active,
            archived: self.archived,
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            limit: self.limit,
            budget: self.budget,
            created_at: self.created_at,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a evidence notice in status planned.
pub fn new_evidence_notice(code: &str, title: &str) -> EvidenceNotice {
    EvidenceNotice {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        active: false,
        archived: false,
        starts_at: 0,
        locale: "".to_string(),
        limit: 0,
        budget: 0.0,
        created_at: 0,
        sequence: 0,
        status: EvidenceNoticeStatus::Planned,
    }
}
