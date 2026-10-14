use super::*;

use acme_kit::text;

/// Where a scheduled evidence template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledEvidenceTemplateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledEvidenceTemplateStatus {
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

impl std::fmt::Display for ScheduledEvidenceTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled evidence template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTemplate {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub source: String,
    pub created_at: i64,
    pub cipher: String,
    pub status: ScheduledEvidenceTemplateStatus,
}

impl ScheduledEvidenceTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledEvidenceTemplateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledEvidenceTemplateStatus) -> bool {
        match self.status {
            ScheduledEvidenceTemplateStatus::Planned => {
                match next {
                    ScheduledEvidenceTemplateStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEvidenceTemplateStatus::Scheduled => {
                match next {
                    ScheduledEvidenceTemplateStatus::InProgress | ScheduledEvidenceTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEvidenceTemplateStatus::InProgress => {
                match next {
                    ScheduledEvidenceTemplateStatus::Completed | ScheduledEvidenceTemplateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledEvidenceTemplateStatus::Completed => {
                match next {
                    ScheduledEvidenceTemplateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ScheduledEvidenceTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled evidence template {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quantity as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledEvidenceTemplate {
        ScheduledEvidenceTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            source: self.source.clone(),
            created_at: self.created_at,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled evidence template in status planned.
pub fn new_scheduled_evidence_template(code: &str, name: &str) -> ScheduledEvidenceTemplate {
    ScheduledEvidenceTemplate {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        source: "".to_string(),
        created_at: 0,
        cipher: "".to_string(),
        status: ScheduledEvidenceTemplateStatus::Planned,
    }
}
