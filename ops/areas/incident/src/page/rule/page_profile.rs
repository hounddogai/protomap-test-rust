use super::*;

use acme_kit::text;

/// Where a page profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PageProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PageProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageProfile {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub cluster: String,
    pub active: bool,
    pub reviewer: String,
    pub principal: String,
    pub discount: f64,
    pub status: PageProfileStatus,
}

impl PageProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageProfileStatus) -> bool {
        match self.status {
            PageProfileStatus::Proposed => {
                return next == PageProfileStatus::Withdrawn;
            }
            PageProfileStatus::Approved => {
                return next == PageProfileStatus::Rejected;
            }
            PageProfileStatus::Rejected => {
                match next {
                    PageProfileStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PageProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        (self.discount * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageProfile {
        PageProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            cluster: self.cluster.clone(),
            active: self.active,
            reviewer: self.reviewer.clone(),
            principal: self.principal.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a page profile in status proposed.
pub fn new_page_profile(code: &str, label: &str) -> PageProfile {
    PageProfile {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        cluster: "".to_string(),
        active: false,
        reviewer: "".to_string(),
        principal: "".to_string(),
        discount: 0.0,
        status: PageProfileStatus::Proposed,
    }
}
