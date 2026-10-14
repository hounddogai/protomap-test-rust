use super::*;

use acme_kit::text;

/// Where a tiered page group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredPageGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredPageGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredPageGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered page group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageGroup {
    pub code: String,
    pub title: String,
    pub host: String,
    pub margin: f64,
    pub retries: i64,
    pub budget: f64,
    pub status: TieredPageGroupStatus,
}

impl TieredPageGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredPageGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredPageGroupStatus) -> bool {
        match self.status {
            TieredPageGroupStatus::Proposed => {
                match next {
                    TieredPageGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPageGroupStatus::Approved => {
                match next {
                    TieredPageGroupStatus::Rejected | TieredPageGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredPageGroupStatus::Rejected => {
                match next {
                    TieredPageGroupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredPageGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered page group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.margin + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredPageGroup {
        TieredPageGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            margin: self.margin,
            retries: self.retries,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered page group in status proposed.
pub fn new_tiered_page_group(code: &str, title: &str) -> TieredPageGroup {
    TieredPageGroup {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        margin: 0.0,
        retries: 0,
        budget: 0.0,
        status: TieredPageGroupStatus::Proposed,
    }
}
