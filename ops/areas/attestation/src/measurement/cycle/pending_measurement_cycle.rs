use super::*;

use acme_kit::text;

/// Where a pending measurement cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingMeasurementCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingMeasurementCycleStatus {
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

impl std::fmt::Display for PendingMeasurementCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending measurement cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementCycle {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub expires_at: i64,
    pub description: String,
    pub status: PendingMeasurementCycleStatus,
}

impl PendingMeasurementCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingMeasurementCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingMeasurementCycleStatus) -> bool {
        match self.status {
            PendingMeasurementCycleStatus::Draft => {
                return next == PendingMeasurementCycleStatus::Suspended;
            }
            PendingMeasurementCycleStatus::Pending => {
                match next {
                    PendingMeasurementCycleStatus::Active | PendingMeasurementCycleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingMeasurementCycleStatus::Active => {
                match next {
                    PendingMeasurementCycleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingMeasurementCycleStatus::Suspended => {
                match next {
                    PendingMeasurementCycleStatus::Closed => {
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

    pub fn move_to(&mut self, next: PendingMeasurementCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending measurement cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending measurement cycle", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.updated_at as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingMeasurementCycle {
        PendingMeasurementCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            expires_at: self.expires_at,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending measurement cycle in status draft.
pub fn new_pending_measurement_cycle(code: &str, label: &str) -> PendingMeasurementCycle {
    PendingMeasurementCycle {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        expires_at: 0,
        description: "".to_string(),
        status: PendingMeasurementCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_measurement_cycle("code-495", "label-413");
        assert_eq!(item.status, PendingMeasurementCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_pending_measurement_cycle("code-495", "label-413");
        if item.move_to(PendingMeasurementCycleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingMeasurementCycleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_measurement_cycle("code-495", "label-413");
        item.status = PendingMeasurementCycleStatus::Closed;
        assert!(
            item.move_to(PendingMeasurementCycleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
