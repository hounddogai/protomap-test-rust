use super::*;

use acme_kit::text;

/// Where a data export claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DataExportClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for DataExportClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportClaim {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub attempts: i64,
    pub currency: String,
    pub expires_at: i64,
    pub cluster: String,
    pub owner_id: String,
    pub duration_minutes: i64,
    pub channel: String,
    pub floor_value: f64,
    pub status: DataExportClaimStatus,
}

impl DataExportClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportClaimStatus) -> bool {
        match self.status {
            DataExportClaimStatus::New => {
                match next {
                    DataExportClaimStatus::Done | DataExportClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportClaimStatus::Queued => {
                match next {
                    DataExportClaimStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportClaimStatus::Running => {
                return next == DataExportClaimStatus::Done;
            }
            DataExportClaimStatus::Done => {
                return next == DataExportClaimStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export claim {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ends_at as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DataExportClaim {
        DataExportClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            attempts: self.attempts,
            currency: self.currency.clone(),
            expires_at: self.expires_at,
            cluster: self.cluster.clone(),
            owner_id: self.owner_id.clone(),
            duration_minutes: self.duration_minutes,
            channel: self.channel.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a data export claim in status new.
pub fn new_data_export_claim(code: &str, label: &str) -> DataExportClaim {
    DataExportClaim {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        attempts: 0,
        currency: "".to_string(),
        expires_at: 0,
        cluster: "".to_string(),
        owner_id: "".to_string(),
        duration_minutes: 0,
        channel: "".to_string(),
        floor_value: 0.0,
        status: DataExportClaimStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_data_export_claim("code-568", "label-577");
        assert_eq!(item.status, DataExportClaimStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_data_export_claim("code-568", "label-577");
        if item.move_to(DataExportClaimStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportClaimStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_data_export_claim("code-568", "label-577");
        item.status = DataExportClaimStatus::Failed;
        assert!(
            item.move_to(DataExportClaimStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
