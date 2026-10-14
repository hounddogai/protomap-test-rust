use super::*;

use acme_kit::text;

/// Where a identity variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IdentityVarianceStatus {
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

impl std::fmt::Display for IdentityVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a identity variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityVariance {
    pub code: String,
    pub label: String,
    pub region: String,
    pub created_at: i64,
    pub tags: Vec<String>,
    pub sequence: i64,
    pub fingerprint: String,
    pub quantity: i64,
    pub version: i64,
    pub status: IdentityVarianceStatus,
}

impl IdentityVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityVarianceStatus) -> bool {
        match self.status {
            IdentityVarianceStatus::Planned => {
                return next == IdentityVarianceStatus::Scheduled || next == IdentityVarianceStatus::Completed;
            }
            IdentityVarianceStatus::Scheduled => {
                return next == IdentityVarianceStatus::InProgress;
            }
            IdentityVarianceStatus::InProgress => {
                return next == IdentityVarianceStatus::Completed || next == IdentityVarianceStatus::Cancelled;
            }
            IdentityVarianceStatus::Completed => {
                match next {
                    IdentityVarianceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: IdentityVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "identity variance", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.version as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityVariance {
        IdentityVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            created_at: self.created_at,
            tags: self.tags.clone(),
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a identity variance in status planned.
pub fn new_identity_variance(code: &str, label: &str) -> IdentityVariance {
    IdentityVariance {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        created_at: 0,
        tags: Vec::new(),
        sequence: 0,
        fingerprint: "".to_string(),
        quantity: 0,
        version: 0,
        status: IdentityVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_identity_variance("code-309", "label-901");
        assert_eq!(item.status, IdentityVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_identity_variance("code-309", "label-901");
        if item.move_to(IdentityVarianceStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityVarianceStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_identity_variance("code-309", "label-901");
        item.status = IdentityVarianceStatus::Cancelled;
        assert!(
            item.move_to(IdentityVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
