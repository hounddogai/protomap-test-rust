use super::*;

use acme_kit::text;

/// Where a alert mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AlertMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AlertMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertMapping {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub ratio: f64,
    pub quota_units: i64,
    pub revision: i64,
    pub channel: String,
    pub starts_at: i64,
    pub status: AlertMappingStatus,
}

impl AlertMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertMappingStatus) -> bool {
        match self.status {
            AlertMappingStatus::Proposed => {
                match next {
                    AlertMappingStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertMappingStatus::Approved => {
                return next == AlertMappingStatus::Withdrawn;
            }
            AlertMappingStatus::Rejected => {
                return next == AlertMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        (self.ratio * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertMapping {
        AlertMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            ratio: self.ratio,
            quota_units: self.quota_units,
            revision: self.revision,
            channel: self.channel.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a alert mapping in status proposed.
pub fn new_alert_mapping(code: &str, title: &str) -> AlertMapping {
    AlertMapping {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        ratio: 0.0,
        quota_units: 0,
        revision: 0,
        channel: "".to_string(),
        starts_at: 0,
        status: AlertMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_alert_mapping("code-417", "title-802");
        assert_eq!(item.status, AlertMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_alert_mapping("code-417", "title-802");
        if item.move_to(AlertMappingStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertMappingStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_alert_mapping("code-417", "title-802");
        item.status = AlertMappingStatus::Withdrawn;
        assert!(
            item.move_to(AlertMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
