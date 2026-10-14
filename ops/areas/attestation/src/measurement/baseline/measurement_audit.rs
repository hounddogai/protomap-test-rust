use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a measurement audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MeasurementAuditStatus {
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

impl std::fmt::Display for MeasurementAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement audit as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAudit {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub attributes: BTreeMap<String, String>,
    pub rotation_days: i64,
    pub host: String,
    pub duration_minutes: i64,
    pub phone: String,
    pub source: String,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub status: MeasurementAuditStatus,
}

impl MeasurementAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementAuditStatus) -> bool {
        match self.status {
            MeasurementAuditStatus::Draft => {
                match next {
                    MeasurementAuditStatus::Pending | MeasurementAuditStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementAuditStatus::Pending => {
                return next == MeasurementAuditStatus::Active || next == MeasurementAuditStatus::Suspended;
            }
            MeasurementAuditStatus::Active => {
                match next {
                    MeasurementAuditStatus::Suspended | MeasurementAuditStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementAuditStatus::Suspended => {
                match next {
                    MeasurementAuditStatus::Closed => {
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

    pub fn move_to(&mut self, next: MeasurementAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement audit", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn projected_value(&self) -> f64 {
        ((self.score - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementAudit {
        MeasurementAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            attributes: self.attributes.clone(),
            rotation_days: self.rotation_days,
            host: self.host.clone(),
            duration_minutes: self.duration_minutes,
            phone: self.phone.clone(),
            source: self.source.clone(),
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a measurement audit in status draft.
pub fn new_measurement_audit(code: &str, name: &str) -> MeasurementAudit {
    MeasurementAudit {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        attributes: BTreeMap::new(),
        rotation_days: 0,
        host: "".to_string(),
        duration_minutes: 0,
        phone: "".to_string(),
        source: "".to_string(),
        expires_at: 0,
        tags: Vec::new(),
        status: MeasurementAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_measurement_audit("code-99", "name-547");
        assert_eq!(item.status, MeasurementAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_measurement_audit("code-99", "name-547");
        if item.move_to(MeasurementAuditStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementAuditStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_measurement_audit("code-99", "name-547");
        item.status = MeasurementAuditStatus::Closed;
        assert!(
            item.move_to(MeasurementAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
