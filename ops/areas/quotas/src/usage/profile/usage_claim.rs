use super::*;

use acme_kit::text;

/// Where a usage claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl UsageClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for UsageClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageClaim {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub amount: f64,
    pub archived: bool,
    pub country: String,
    pub currency: String,
    pub label: String,
    pub audience: String,
    pub updated_at: i64,
    pub status: UsageClaimStatus,
}

impl UsageClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageClaimStatus) -> bool {
        match self.status {
            UsageClaimStatus::Created => {
                match next {
                    UsageClaimStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageClaimStatus::Validated => {
                return next == UsageClaimStatus::Published;
            }
            UsageClaimStatus::Published => {
                return next == UsageClaimStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "usage claim", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageClaim {
        UsageClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            amount: self.amount,
            archived: self.archived,
            country: self.country.clone(),
            currency: self.currency.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a usage claim in status created.
pub fn new_usage_claim(code: &str, title: &str) -> UsageClaim {
    UsageClaim {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        amount: 0.0,
        archived: false,
        country: "".to_string(),
        currency: "".to_string(),
        label: "".to_string(),
        audience: "".to_string(),
        updated_at: 0,
        status: UsageClaimStatus::Created,
    }
}
