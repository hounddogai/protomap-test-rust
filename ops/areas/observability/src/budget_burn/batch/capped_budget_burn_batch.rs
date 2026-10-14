use super::*;

use acme_kit::text;

/// Where a capped budget burn batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedBudgetBurnBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedBudgetBurnBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedBudgetBurnBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped budget burn batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnBatch {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub retries: i64,
    pub version: i64,
    pub discount: f64,
    pub checksum: String,
    pub created_at: i64,
    pub category: String,
    pub quantity: i64,
    pub rotation_days: i64,
    pub status: CappedBudgetBurnBatchStatus,
}

impl CappedBudgetBurnBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedBudgetBurnBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedBudgetBurnBatchStatus) -> bool {
        match self.status {
            CappedBudgetBurnBatchStatus::Requested => {
                return next == CappedBudgetBurnBatchStatus::Granted || next == CappedBudgetBurnBatchStatus::Revoked;
            }
            CappedBudgetBurnBatchStatus::Granted => {
                return next == CappedBudgetBurnBatchStatus::Revoked || next == CappedBudgetBurnBatchStatus::Expired;
            }
            CappedBudgetBurnBatchStatus::Revoked => {
                return next == CappedBudgetBurnBatchStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedBudgetBurnBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped budget burn batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.retries as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedBudgetBurnBatch {
        CappedBudgetBurnBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            retries: self.retries,
            version: self.version,
            discount: self.discount,
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            category: self.category.clone(),
            quantity: self.quantity,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped budget burn batch in status requested.
pub fn new_capped_budget_burn_batch(code: &str, label: &str) -> CappedBudgetBurnBatch {
    CappedBudgetBurnBatch {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        retries: 0,
        version: 0,
        discount: 0.0,
        checksum: "".to_string(),
        created_at: 0,
        category: "".to_string(),
        quantity: 0,
        rotation_days: 0,
        status: CappedBudgetBurnBatchStatus::Requested,
    }
}
