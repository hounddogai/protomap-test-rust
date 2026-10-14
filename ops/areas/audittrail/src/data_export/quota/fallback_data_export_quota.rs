use super::*;

use acme_kit::text;

/// Where a fallback data export quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDataExportQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackDataExportQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackDataExportQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback data export quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportQuota {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub cost: f64,
    pub quota_units: i64,
    pub region: String,
    pub title: String,
    pub status: FallbackDataExportQuotaStatus,
}

impl FallbackDataExportQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDataExportQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDataExportQuotaStatus) -> bool {
        match self.status {
            FallbackDataExportQuotaStatus::Requested => {
                return next == FallbackDataExportQuotaStatus::Revoked
                    || next == FallbackDataExportQuotaStatus::Expired;
            }
            FallbackDataExportQuotaStatus::Granted => {
                match next {
                    FallbackDataExportQuotaStatus::Revoked | FallbackDataExportQuotaStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDataExportQuotaStatus::Revoked => {
                match next {
                    FallbackDataExportQuotaStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackDataExportQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback data export quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        (self.rate * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackDataExportQuota {
        FallbackDataExportQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            cost: self.cost,
            quota_units: self.quota_units,
            region: self.region.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a fallback data export quota in status requested.
pub fn new_fallback_data_export_quota(code: &str, label: &str) -> FallbackDataExportQuota {
    FallbackDataExportQuota {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        cost: 0.0,
        quota_units: 0,
        region: "".to_string(),
        title: "".to_string(),
        status: FallbackDataExportQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_data_export_quota("code-993", "label-513");
        assert_eq!(item.status, FallbackDataExportQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_fallback_data_export_quota("code-993", "label-513");
        if item.move_to(FallbackDataExportQuotaStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDataExportQuotaStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_data_export_quota("code-993", "label-513");
        item.status = FallbackDataExportQuotaStatus::Expired;
        assert!(
            item.move_to(FallbackDataExportQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
