use super::*;

use acme_kit::text;

/// Where a default credential metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultCredentialMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultCredentialMetricStatus {
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

impl std::fmt::Display for DefaultCredentialMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default credential metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultCredentialMetric {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub ends_at: i64,
    pub source: String,
    pub cluster: String,
    pub phone: String,
    pub country: String,
    pub status: DefaultCredentialMetricStatus,
}

impl DefaultCredentialMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultCredentialMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultCredentialMetricStatus) -> bool {
        match self.status {
            DefaultCredentialMetricStatus::Open => {
                return next == DefaultCredentialMetricStatus::InReview
                    || next == DefaultCredentialMetricStatus::Reopened;
            }
            DefaultCredentialMetricStatus::InReview => {
                match next {
                    DefaultCredentialMetricStatus::Reopened | DefaultCredentialMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCredentialMetricStatus::Resolved => {
                match next {
                    DefaultCredentialMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultCredentialMetricStatus::Reopened => {
                match next {
                    DefaultCredentialMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultCredentialMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default credential metric {} cannot move from {} to {}",
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

    pub fn weighted_value(&self) -> f64 {
        ((self.quantity as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultCredentialMetric {
        DefaultCredentialMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            ends_at: self.ends_at,
            source: self.source.clone(),
            cluster: self.cluster.clone(),
            phone: self.phone.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default credential metric in status open.
pub fn new_default_credential_metric(code: &str, label: &str) -> DefaultCredentialMetric {
    DefaultCredentialMetric {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        ends_at: 0,
        source: "".to_string(),
        cluster: "".to_string(),
        phone: "".to_string(),
        country: "".to_string(),
        status: DefaultCredentialMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_credential_metric("code-482", "label-539");
        assert_eq!(item.status, DefaultCredentialMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_default_credential_metric("code-482", "label-539");
        if item.move_to(DefaultCredentialMetricStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultCredentialMetricStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_credential_metric("code-482", "label-539");
        item.status = DefaultCredentialMetricStatus::Archived;
        assert!(
            item.move_to(DefaultCredentialMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
