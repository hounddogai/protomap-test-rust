use super::*;

use acme_kit::text;

/// Where a peering quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PeeringQuotaStatus {
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

impl std::fmt::Display for PeeringQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a peering quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringQuota {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub expires_at: i64,
    pub external_ref: String,
    pub updated_at: i64,
    pub batch_size: i64,
    pub status: PeeringQuotaStatus,
}

impl PeeringQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringQuotaStatus) -> bool {
        match self.status {
            PeeringQuotaStatus::New => {
                return next == PeeringQuotaStatus::Failed;
            }
            PeeringQuotaStatus::Queued => {
                match next {
                    PeeringQuotaStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringQuotaStatus::Running => {
                match next {
                    PeeringQuotaStatus::Done | PeeringQuotaStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringQuotaStatus::Done => {
                return next == PeeringQuotaStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering quota {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.expires_at as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringQuota {
        PeeringQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            updated_at: self.updated_at,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a peering quota in status new.
pub fn new_peering_quota(code: &str, label: &str) -> PeeringQuota {
    PeeringQuota {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        expires_at: 0,
        external_ref: "".to_string(),
        updated_at: 0,
        batch_size: 0,
        status: PeeringQuotaStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_peering_quota("code-466", "label-62");
        assert_eq!(item.status, PeeringQuotaStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_peering_quota("code-466", "label-62");
        if item.move_to(PeeringQuotaStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringQuotaStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_peering_quota("code-466", "label-62");
        item.status = PeeringQuotaStatus::Failed;
        assert!(item.move_to(PeeringQuotaStatus::New).is_err(), "expected move to fail");
    }
}
