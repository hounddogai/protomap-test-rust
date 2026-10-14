use super::*;

use acme_kit::text;

/// Where a measurement record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementRecordStatus {
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

impl std::fmt::Display for MeasurementRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement record as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRecord {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub cluster: String,
    pub limit: i64,
    pub source: String,
    pub cipher: String,
    pub timezone: String,
    pub issuer: String,
    pub status: MeasurementRecordStatus,
}

impl MeasurementRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementRecordStatus) -> bool {
        match self.status {
            MeasurementRecordStatus::Idle => {
                return next == MeasurementRecordStatus::Draining;
            }
            MeasurementRecordStatus::Warming => {
                match next {
                    MeasurementRecordStatus::Ready | MeasurementRecordStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementRecordStatus::Ready => {
                return next == MeasurementRecordStatus::Draining || next == MeasurementRecordStatus::Stopped;
            }
            MeasurementRecordStatus::Draining => {
                match next {
                    MeasurementRecordStatus::Stopped => {
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

    pub fn move_to(&mut self, next: MeasurementRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "measurement record", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> MeasurementRecord {
        MeasurementRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            cluster: self.cluster.clone(),
            limit: self.limit,
            source: self.source.clone(),
            cipher: self.cipher.clone(),
            timezone: self.timezone.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a measurement record in status idle.
pub fn new_measurement_record(code: &str, label: &str) -> MeasurementRecord {
    MeasurementRecord {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        cluster: "".to_string(),
        limit: 0,
        source: "".to_string(),
        cipher: "".to_string(),
        timezone: "".to_string(),
        issuer: "".to_string(),
        status: MeasurementRecordStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_measurement_record("code-948", "label-124");
        assert_eq!(item.status, MeasurementRecordStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_measurement_record("code-948", "label-124");
        if item.move_to(MeasurementRecordStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementRecordStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_measurement_record("code-948", "label-124");
        item.status = MeasurementRecordStatus::Stopped;
        assert!(
            item.move_to(MeasurementRecordStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
