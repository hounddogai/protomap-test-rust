use super::*;

use acme_kit::text;

/// Where a tenant draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TenantDraftStatus {
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

impl std::fmt::Display for TenantDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant draft as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantDraft {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub region: String,
    pub ceiling: f64,
    pub burst: i64,
    pub quota_units: i64,
    pub status: TenantDraftStatus,
}

impl TenantDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantDraftStatus) -> bool {
        match self.status {
            TenantDraftStatus::Draft => {
                match next {
                    TenantDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantDraftStatus::Pending => {
                return next == TenantDraftStatus::Suspended || next == TenantDraftStatus::Closed;
            }
            TenantDraftStatus::Active => {
                match next {
                    TenantDraftStatus::Suspended | TenantDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantDraftStatus::Suspended => {
                match next {
                    TenantDraftStatus::Closed => {
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

    pub fn move_to(&mut self, next: TenantDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.burst as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantDraft {
        TenantDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            region: self.region.clone(),
            ceiling: self.ceiling,
            burst: self.burst,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tenant draft in status draft.
pub fn new_tenant_draft(code: &str, name: &str) -> TenantDraft {
    TenantDraft {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        region: "".to_string(),
        ceiling: 0.0,
        burst: 0,
        quota_units: 0,
        status: TenantDraftStatus::Draft,
    }
}
