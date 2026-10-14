use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a alert claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AlertClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for AlertClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertClaim {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub cost: f64,
    pub retries: i64,
    pub summary: String,
    pub attributes: BTreeMap<String, String>,
    pub locale: String,
    pub score: f64,
    pub status: AlertClaimStatus,
}

impl AlertClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertClaimStatus) -> bool {
        match self.status {
            AlertClaimStatus::Open => {
                return next == AlertClaimStatus::Reopened || next == AlertClaimStatus::Archived;
            }
            AlertClaimStatus::InReview => {
                match next {
                    AlertClaimStatus::Reopened | AlertClaimStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertClaimStatus::Resolved => {
                return next == AlertClaimStatus::Reopened;
            }
            AlertClaimStatus::Reopened => {
                match next {
                    AlertClaimStatus::Archived => {
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

    pub fn move_to(&mut self, next: AlertClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn adjusted_total(&self) -> f64 {
        ((self.cost - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertClaim {
        AlertClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            cost: self.cost,
            retries: self.retries,
            summary: self.summary.clone(),
            attributes: self.attributes.clone(),
            locale: self.locale.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a alert claim in status open.
pub fn new_alert_claim(code: &str, label: &str) -> AlertClaim {
    AlertClaim {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        cost: 0.0,
        retries: 0,
        summary: "".to_string(),
        attributes: BTreeMap::new(),
        locale: "".to_string(),
        score: 0.0,
        status: AlertClaimStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_alert_claim("code-62", "label-811");
        assert_eq!(item.status, AlertClaimStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_alert_claim("code-62", "label-811");
        if item.move_to(AlertClaimStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertClaimStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_alert_claim("code-62", "label-811");
        item.status = AlertClaimStatus::Archived;
        assert!(item.move_to(AlertClaimStatus::Open).is_err(), "expected move to fail");
    }
}
