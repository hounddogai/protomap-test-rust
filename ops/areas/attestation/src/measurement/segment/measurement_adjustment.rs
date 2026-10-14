use super::*;

use acme_kit::text;

/// Where a measurement adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MeasurementAdjustmentStatus {
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

impl std::fmt::Display for MeasurementAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One measurement adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAdjustment {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub floor_value: f64,
    pub attempts: i64,
    pub starts_at: i64,
    pub updated_at: i64,
    pub unit: String,
    pub created_at: i64,
    pub status: MeasurementAdjustmentStatus,
}

impl MeasurementAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementAdjustmentStatus) -> bool {
        match self.status {
            MeasurementAdjustmentStatus::Draft => {
                match next {
                    MeasurementAdjustmentStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementAdjustmentStatus::Pending => {
                match next {
                    MeasurementAdjustmentStatus::Suspended | MeasurementAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementAdjustmentStatus::Active => {
                match next {
                    MeasurementAdjustmentStatus::Suspended | MeasurementAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementAdjustmentStatus::Suspended => {
                match next {
                    MeasurementAdjustmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: MeasurementAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement adjustment {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        (self.floor_value * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementAdjustment {
        MeasurementAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            floor_value: self.floor_value,
            attempts: self.attempts,
            starts_at: self.starts_at,
            updated_at: self.updated_at,
            unit: self.unit.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a measurement adjustment in status draft.
pub fn new_measurement_adjustment(code: &str, name: &str) -> MeasurementAdjustment {
    MeasurementAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        floor_value: 0.0,
        attempts: 0,
        starts_at: 0,
        updated_at: 0,
        unit: "".to_string(),
        created_at: 0,
        status: MeasurementAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_measurement_adjustment("code-413", "name-448");
        assert_eq!(item.status, MeasurementAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_measurement_adjustment("code-413", "name-448");
        if item.move_to(MeasurementAdjustmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementAdjustmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_measurement_adjustment("code-413", "name-448");
        item.status = MeasurementAdjustmentStatus::Closed;
        assert!(
            item.move_to(MeasurementAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
