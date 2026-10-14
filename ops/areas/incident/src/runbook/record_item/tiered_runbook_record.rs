use super::*;

use acme_kit::text;

/// Where a tiered runbook record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRunbookRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredRunbookRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredRunbookRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered runbook record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookRecord {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub quantity: i64,
    pub sequence: i64,
    pub rate: f64,
    pub ttl_seconds: i64,
    pub tenant_id: String,
    pub status: TieredRunbookRecordStatus,
}

impl TieredRunbookRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRunbookRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRunbookRecordStatus) -> bool {
        match self.status {
            TieredRunbookRecordStatus::Created => {
                match next {
                    TieredRunbookRecordStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRunbookRecordStatus::Validated => {
                match next {
                    TieredRunbookRecordStatus::Published | TieredRunbookRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRunbookRecordStatus::Published => {
                return next == TieredRunbookRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredRunbookRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered runbook record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered runbook record", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.sequence as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRunbookRecord {
        TieredRunbookRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            quantity: self.quantity,
            sequence: self.sequence,
            rate: self.rate,
            ttl_seconds: self.ttl_seconds,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a tiered runbook record in status created.
pub fn new_tiered_runbook_record(code: &str, title: &str) -> TieredRunbookRecord {
    TieredRunbookRecord {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        external_ref: "".to_string(),
        quantity: 0,
        sequence: 0,
        rate: 0.0,
        ttl_seconds: 0,
        tenant_id: "".to_string(),
        status: TieredRunbookRecordStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_runbook_record("code-644", "title-960");
        assert_eq!(item.status, TieredRunbookRecordStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_tiered_runbook_record("code-644", "title-960");
        if item.move_to(TieredRunbookRecordStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRunbookRecordStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_runbook_record("code-644", "title-960");
        item.status = TieredRunbookRecordStatus::Retired;
        assert!(
            item.move_to(TieredRunbookRecordStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
