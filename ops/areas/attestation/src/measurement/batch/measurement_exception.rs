use super::*;

use acme_kit::text;

/// Where a measurement exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MeasurementExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MeasurementExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement exception as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementException {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub title: String,
    pub updated_at: i64,
    pub namespace: String,
    pub starts_at: i64,
    pub cluster: String,
    pub score: f64,
    pub burst: i64,
    pub timezone: String,
    pub status: MeasurementExceptionStatus,
}

impl MeasurementException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementExceptionStatus) -> bool {
        match self.status {
            MeasurementExceptionStatus::Created => {
                match next {
                    MeasurementExceptionStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementExceptionStatus::Validated => {
                match next {
                    MeasurementExceptionStatus::Published | MeasurementExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementExceptionStatus::Published => {
                match next {
                    MeasurementExceptionStatus::Retired => {
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

    pub fn move_to(&mut self, next: MeasurementExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement exception {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.updated_at as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementException {
        MeasurementException {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            namespace: self.namespace.clone(),
            starts_at: self.starts_at,
            cluster: self.cluster.clone(),
            score: self.score,
            burst: self.burst,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a measurement exception in status created.
pub fn new_measurement_exception(code: &str, name: &str) -> MeasurementException {
    MeasurementException {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        title: "".to_string(),
        updated_at: 0,
        namespace: "".to_string(),
        starts_at: 0,
        cluster: "".to_string(),
        score: 0.0,
        burst: 0,
        timezone: "".to_string(),
        status: MeasurementExceptionStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_measurement_exception("code-517", "name-957");
        assert_eq!(item.status, MeasurementExceptionStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_measurement_exception("code-517", "name-957");
        if item.move_to(MeasurementExceptionStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementExceptionStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_measurement_exception("code-517", "name-957");
        item.status = MeasurementExceptionStatus::Retired;
        assert!(
            item.move_to(MeasurementExceptionStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
