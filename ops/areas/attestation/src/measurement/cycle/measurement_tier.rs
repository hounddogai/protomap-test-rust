use super::*;

use acme_kit::text;

/// Where a measurement tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for MeasurementTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement tier as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTier {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub fingerprint: String,
    pub batch_size: i64,
    pub created_at: i64,
    pub namespace: String,
    pub floor_value: f64,
    pub amount: f64,
    pub issuer: String,
    pub status: MeasurementTierStatus,
}

impl MeasurementTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementTierStatus) -> bool {
        match self.status {
            MeasurementTierStatus::Idle => {
                return next == MeasurementTierStatus::Ready || next == MeasurementTierStatus::Draining;
            }
            MeasurementTierStatus::Warming => {
                match next {
                    MeasurementTierStatus::Ready | MeasurementTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementTierStatus::Ready => {
                match next {
                    MeasurementTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementTierStatus::Draining => {
                return next == MeasurementTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement tier {} cannot move from {} to {}",
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
        format!("{} {}", "measurement tier", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.sequence as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementTier {
        MeasurementTier {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            batch_size: self.batch_size,
            created_at: self.created_at,
            namespace: self.namespace.clone(),
            floor_value: self.floor_value,
            amount: self.amount,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a measurement tier in status idle.
pub fn new_measurement_tier(code: &str, name: &str) -> MeasurementTier {
    MeasurementTier {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        fingerprint: "".to_string(),
        batch_size: 0,
        created_at: 0,
        namespace: "".to_string(),
        floor_value: 0.0,
        amount: 0.0,
        issuer: "".to_string(),
        status: MeasurementTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_measurement_tier("code-297", "name-843");
        assert_eq!(item.status, MeasurementTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_measurement_tier("code-297", "name-843");
        if item.move_to(MeasurementTierStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementTierStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_measurement_tier("code-297", "name-843");
        item.status = MeasurementTierStatus::Stopped;
        assert!(
            item.move_to(MeasurementTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
