use super::*;

use acme_kit::text;

/// Where a credential variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CredentialVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CredentialVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential variance as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialVariance {
    pub code: String,
    pub title: String,
    pub country: String,
    pub rate: f64,
    pub locale: String,
    pub name: String,
    pub phone: String,
    pub rotation_days: i64,
    pub limit: i64,
    pub segment: String,
    pub status: CredentialVarianceStatus,
}

impl CredentialVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialVarianceStatus) -> bool {
        match self.status {
            CredentialVarianceStatus::Planned => {
                return next == CredentialVarianceStatus::Scheduled;
            }
            CredentialVarianceStatus::Scheduled => {
                match next {
                    CredentialVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialVarianceStatus::InProgress => {
                match next {
                    CredentialVarianceStatus::Completed | CredentialVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialVarianceStatus::Completed => {
                match next {
                    CredentialVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CredentialVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential variance", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.limit as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialVariance {
        CredentialVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            rate: self.rate,
            locale: self.locale.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            rotation_days: self.rotation_days,
            limit: self.limit,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a credential variance in status planned.
pub fn new_credential_variance(code: &str, title: &str) -> CredentialVariance {
    CredentialVariance {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        rate: 0.0,
        locale: "".to_string(),
        name: "".to_string(),
        phone: "".to_string(),
        rotation_days: 0,
        limit: 0,
        segment: "".to_string(),
        status: CredentialVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_credential_variance("code-276", "title-112");
        assert_eq!(item.status, CredentialVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_credential_variance("code-276", "title-112");
        if item.move_to(CredentialVarianceStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialVarianceStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_credential_variance("code-276", "title-112");
        item.status = CredentialVarianceStatus::Cancelled;
        assert!(
            item.move_to(CredentialVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
