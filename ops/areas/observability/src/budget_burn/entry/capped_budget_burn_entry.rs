use super::*;

use acme_kit::text;

/// Where a capped budget burn entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedBudgetBurnEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedBudgetBurnEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedBudgetBurnEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped budget burn entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnEntry {
    pub code: String,
    pub title: String,
    pub name: String,
    pub duration_minutes: i64,
    pub reviewer: String,
    pub starts_at: i64,
    pub discount: f64,
    pub namespace: String,
    pub category: String,
    pub issuer: String,
    pub status: CappedBudgetBurnEntryStatus,
}

impl CappedBudgetBurnEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedBudgetBurnEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedBudgetBurnEntryStatus) -> bool {
        match self.status {
            CappedBudgetBurnEntryStatus::Created => {
                match next {
                    CappedBudgetBurnEntryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnEntryStatus::Validated => {
                match next {
                    CappedBudgetBurnEntryStatus::Published | CappedBudgetBurnEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnEntryStatus::Published => {
                match next {
                    CappedBudgetBurnEntryStatus::Retired => {
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

    pub fn move_to(&mut self, next: CappedBudgetBurnEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped budget burn entry {} cannot move from {} to {}",
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
        format!("{} {}", "capped budget burn entry", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.starts_at as f64 - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedBudgetBurnEntry {
        CappedBudgetBurnEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            reviewer: self.reviewer.clone(),
            starts_at: self.starts_at,
            discount: self.discount,
            namespace: self.namespace.clone(),
            category: self.category.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a capped budget burn entry in status created.
pub fn new_capped_budget_burn_entry(code: &str, title: &str) -> CappedBudgetBurnEntry {
    CappedBudgetBurnEntry {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        duration_minutes: 0,
        reviewer: "".to_string(),
        starts_at: 0,
        discount: 0.0,
        namespace: "".to_string(),
        category: "".to_string(),
        issuer: "".to_string(),
        status: CappedBudgetBurnEntryStatus::Created,
    }
}
