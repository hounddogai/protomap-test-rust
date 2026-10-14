use super::*;

use acme_kit::text;

/// Where a regional query variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalQueryVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalQueryVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalQueryVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional query variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryVariance {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub weight: f64,
    pub host: String,
    pub region: String,
    pub updated_at: i64,
    pub status: RegionalQueryVarianceStatus,
}

impl RegionalQueryVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalQueryVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalQueryVarianceStatus) -> bool {
        match self.status {
            RegionalQueryVarianceStatus::Requested => {
                return next == RegionalQueryVarianceStatus::Granted || next == RegionalQueryVarianceStatus::Revoked;
            }
            RegionalQueryVarianceStatus::Granted => {
                match next {
                    RegionalQueryVarianceStatus::Revoked | RegionalQueryVarianceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalQueryVarianceStatus::Revoked => {
                match next {
                    RegionalQueryVarianceStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalQueryVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional query variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional query variance", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.revision as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalQueryVariance {
        RegionalQueryVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            weight: self.weight,
            host: self.host.clone(),
            region: self.region.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional query variance in status requested.
pub fn new_regional_query_variance(code: &str, label: &str) -> RegionalQueryVariance {
    RegionalQueryVariance {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        weight: 0.0,
        host: "".to_string(),
        region: "".to_string(),
        updated_at: 0,
        status: RegionalQueryVarianceStatus::Requested,
    }
}
