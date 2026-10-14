use super::*;

use acme_kit::text;

/// Where a measurement ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementLedgerStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MeasurementLedgerStatus {
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

impl std::fmt::Display for MeasurementLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement ledger as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementLedger {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub ends_at: i64,
    pub approved: bool,
    pub cipher: String,
    pub segment: String,
    pub active: bool,
    pub summary: String,
    pub status: MeasurementLedgerStatus,
}

impl MeasurementLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementLedgerStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementLedgerStatus) -> bool {
        match self.status {
            MeasurementLedgerStatus::Draft => {
                return next == MeasurementLedgerStatus::Suspended;
            }
            MeasurementLedgerStatus::Pending => {
                return next == MeasurementLedgerStatus::Active || next == MeasurementLedgerStatus::Closed;
            }
            MeasurementLedgerStatus::Active => {
                match next {
                    MeasurementLedgerStatus::Suspended | MeasurementLedgerStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementLedgerStatus::Suspended => {
                return next == MeasurementLedgerStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement ledger", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> MeasurementLedger {
        MeasurementLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            ends_at: self.ends_at,
            approved: self.approved,
            cipher: self.cipher.clone(),
            segment: self.segment.clone(),
            active: self.active,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a measurement ledger in status draft.
pub fn new_measurement_ledger(code: &str, name: &str) -> MeasurementLedger {
    MeasurementLedger {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        ends_at: 0,
        approved: false,
        cipher: "".to_string(),
        segment: "".to_string(),
        active: false,
        summary: "".to_string(),
        status: MeasurementLedgerStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_measurement_ledger("code-625", "name-121");
        assert_eq!(item.status, MeasurementLedgerStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_measurement_ledger("code-625", "name-121");
        if item.move_to(MeasurementLedgerStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementLedgerStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_measurement_ledger("code-625", "name-121");
        item.status = MeasurementLedgerStatus::Closed;
        assert!(
            item.move_to(MeasurementLedgerStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
