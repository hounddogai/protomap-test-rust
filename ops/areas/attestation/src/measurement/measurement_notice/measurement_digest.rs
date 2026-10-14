use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a measurement digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MeasurementDigestStatus {
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

impl std::fmt::Display for MeasurementDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementDigest {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub retries: i64,
    pub cipher: String,
    pub batch_size: i64,
    pub port: i64,
    pub status: MeasurementDigestStatus,
}

impl MeasurementDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementDigestStatus) -> bool {
        match self.status {
            MeasurementDigestStatus::Draft => {
                return next == MeasurementDigestStatus::Pending || next == MeasurementDigestStatus::Closed;
            }
            MeasurementDigestStatus::Pending => {
                return next == MeasurementDigestStatus::Closed;
            }
            MeasurementDigestStatus::Active => {
                return next == MeasurementDigestStatus::Suspended || next == MeasurementDigestStatus::Closed;
            }
            MeasurementDigestStatus::Suspended => {
                match next {
                    MeasurementDigestStatus::Closed => {
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

    pub fn move_to(&mut self, next: MeasurementDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.batch_size as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementDigest {
        MeasurementDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            retries: self.retries,
            cipher: self.cipher.clone(),
            batch_size: self.batch_size,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a measurement digest in status draft.
pub fn new_measurement_digest(code: &str, name: &str) -> MeasurementDigest {
    MeasurementDigest {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        retries: 0,
        cipher: "".to_string(),
        batch_size: 0,
        port: 0,
        status: MeasurementDigestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_measurement_digest("code-967", "name-565");
        assert_eq!(item.status, MeasurementDigestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_measurement_digest("code-967", "name-565");
        if item.move_to(MeasurementDigestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementDigestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_measurement_digest("code-967", "name-565");
        item.status = MeasurementDigestStatus::Closed;
        assert!(
            item.move_to(MeasurementDigestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
