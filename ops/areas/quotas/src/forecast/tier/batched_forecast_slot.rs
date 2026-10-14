use super::*;

use acme_kit::text;

/// Where a batched forecast slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedForecastSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedForecastSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedForecastSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched forecast slot as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastSlot {
    pub code: String,
    pub label: String,
    pub name: String,
    pub discount: f64,
    pub burst: i64,
    pub revision: i64,
    pub sequence: i64,
    pub checksum: String,
    pub created_at: i64,
    pub rotation_days: i64,
    pub status: BatchedForecastSlotStatus,
}

impl BatchedForecastSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedForecastSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedForecastSlotStatus) -> bool {
        match self.status {
            BatchedForecastSlotStatus::Requested => {
                match next {
                    BatchedForecastSlotStatus::Granted | BatchedForecastSlotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedForecastSlotStatus::Granted => {
                match next {
                    BatchedForecastSlotStatus::Revoked | BatchedForecastSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedForecastSlotStatus::Revoked => {
                return next == BatchedForecastSlotStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedForecastSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched forecast slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched forecast slot", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.revision as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedForecastSlot {
        BatchedForecastSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            discount: self.discount,
            burst: self.burst,
            revision: self.revision,
            sequence: self.sequence,
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched forecast slot in status requested.
pub fn new_batched_forecast_slot(code: &str, label: &str) -> BatchedForecastSlot {
    BatchedForecastSlot {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        discount: 0.0,
        burst: 0,
        revision: 0,
        sequence: 0,
        checksum: "".to_string(),
        created_at: 0,
        rotation_days: 0,
        status: BatchedForecastSlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_forecast_slot("code-90", "label-132");
        assert_eq!(item.status, BatchedForecastSlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_batched_forecast_slot("code-90", "label-132");
        if item.move_to(BatchedForecastSlotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedForecastSlotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_forecast_slot("code-90", "label-132");
        item.status = BatchedForecastSlotStatus::Expired;
        assert!(
            item.move_to(BatchedForecastSlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
