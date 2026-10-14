use super::*;

use acme_kit::text;

/// Where a seasonal page quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalPageQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalPageQuotaStatus {
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

impl std::fmt::Display for SeasonalPageQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal page quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageQuota {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub priority: i64,
    pub notes: String,
    pub status: SeasonalPageQuotaStatus,
}

impl SeasonalPageQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalPageQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalPageQuotaStatus) -> bool {
        match self.status {
            SeasonalPageQuotaStatus::Planned => {
                match next {
                    SeasonalPageQuotaStatus::Scheduled | SeasonalPageQuotaStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPageQuotaStatus::Scheduled => {
                match next {
                    SeasonalPageQuotaStatus::Completed | SeasonalPageQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPageQuotaStatus::InProgress => {
                match next {
                    SeasonalPageQuotaStatus::Completed | SeasonalPageQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPageQuotaStatus::Completed => {
                return next == SeasonalPageQuotaStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalPageQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal page quota {} cannot move from {} to {}",
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

    pub fn copy(&self) -> SeasonalPageQuota {
        SeasonalPageQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            priority: self.priority,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal page quota in status planned.
pub fn new_seasonal_page_quota(code: &str, label: &str) -> SeasonalPageQuota {
    SeasonalPageQuota {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        priority: 0,
        notes: "".to_string(),
        status: SeasonalPageQuotaStatus::Planned,
    }
}
