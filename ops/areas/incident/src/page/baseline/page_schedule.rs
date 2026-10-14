use super::*;

use acme_kit::text;

/// Where a page schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PageScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PageScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageSchedule {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub score: f64,
    pub external_ref: String,
    pub status: PageScheduleStatus,
}

impl PageSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageScheduleStatus) -> bool {
        match self.status {
            PageScheduleStatus::Proposed => {
                match next {
                    PageScheduleStatus::Approved | PageScheduleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageScheduleStatus::Approved => {
                match next {
                    PageScheduleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageScheduleStatus::Rejected => {
                match next {
                    PageScheduleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PageScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "page schedule", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PageSchedule {
        PageSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            score: self.score,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a page schedule in status proposed.
pub fn new_page_schedule(code: &str, name: &str) -> PageSchedule {
    PageSchedule {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        score: 0.0,
        external_ref: "".to_string(),
        status: PageScheduleStatus::Proposed,
    }
}
