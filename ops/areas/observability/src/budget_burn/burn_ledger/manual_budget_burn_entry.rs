use super::*;

use acme_kit::text;

/// Where a manual budget burn entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualBudgetBurnEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualBudgetBurnEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualBudgetBurnEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual budget burn entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnEntry {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub ttl_seconds: i64,
    pub created_at: i64,
    pub channel: String,
    pub rotation_days: i64,
    pub cluster: String,
    pub cipher: String,
    pub key_id: String,
    pub external_ref: String,
    pub status: ManualBudgetBurnEntryStatus,
}

impl ManualBudgetBurnEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualBudgetBurnEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualBudgetBurnEntryStatus) -> bool {
        match self.status {
            ManualBudgetBurnEntryStatus::Created => {
                match next {
                    ManualBudgetBurnEntryStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBudgetBurnEntryStatus::Validated => {
                return next == ManualBudgetBurnEntryStatus::Published || next == ManualBudgetBurnEntryStatus::Retired;
            }
            ManualBudgetBurnEntryStatus::Published => {
                match next {
                    ManualBudgetBurnEntryStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualBudgetBurnEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual budget burn entry {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.created_at as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualBudgetBurnEntry {
        ManualBudgetBurnEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            ttl_seconds: self.ttl_seconds,
            created_at: self.created_at,
            channel: self.channel.clone(),
            rotation_days: self.rotation_days,
            cluster: self.cluster.clone(),
            cipher: self.cipher.clone(),
            key_id: self.key_id.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual budget burn entry in status created.
pub fn new_manual_budget_burn_entry(code: &str, title: &str) -> ManualBudgetBurnEntry {
    ManualBudgetBurnEntry {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        ttl_seconds: 0,
        created_at: 0,
        channel: "".to_string(),
        rotation_days: 0,
        cluster: "".to_string(),
        cipher: "".to_string(),
        key_id: "".to_string(),
        external_ref: "".to_string(),
        status: ManualBudgetBurnEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_manual_budget_burn_entry("code-633", "title-652");
        assert_eq!(item.status, ManualBudgetBurnEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_manual_budget_burn_entry("code-633", "title-652");
        if item.move_to(ManualBudgetBurnEntryStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualBudgetBurnEntryStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_manual_budget_burn_entry("code-633", "title-652");
        item.status = ManualBudgetBurnEntryStatus::Retired;
        assert!(
            item.move_to(ManualBudgetBurnEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
