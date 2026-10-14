use super::*;

use acme_kit::text;

/// Where a capped usage profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedUsageProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedUsageProfileStatus {
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

impl std::fmt::Display for CappedUsageProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped usage profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedUsageProfile {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub rate: f64,
    pub priority: i64,
    pub channel: String,
    pub timezone: String,
    pub fingerprint: String,
    pub key_id: String,
    pub status: CappedUsageProfileStatus,
}

impl CappedUsageProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedUsageProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedUsageProfileStatus) -> bool {
        match self.status {
            CappedUsageProfileStatus::Open => {
                match next {
                    CappedUsageProfileStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedUsageProfileStatus::InReview => {
                match next {
                    CappedUsageProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedUsageProfileStatus::Resolved => {
                match next {
                    CappedUsageProfileStatus::Reopened | CappedUsageProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedUsageProfileStatus::Reopened => {
                match next {
                    CappedUsageProfileStatus::Archived => {
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

    pub fn move_to(&mut self, next: CappedUsageProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped usage profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.rate - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedUsageProfile {
        CappedUsageProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            rate: self.rate,
            priority: self.priority,
            channel: self.channel.clone(),
            timezone: self.timezone.clone(),
            fingerprint: self.fingerprint.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a capped usage profile in status open.
pub fn new_capped_usage_profile(code: &str, label: &str) -> CappedUsageProfile {
    CappedUsageProfile {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        rate: 0.0,
        priority: 0,
        channel: "".to_string(),
        timezone: "".to_string(),
        fingerprint: "".to_string(),
        key_id: "".to_string(),
        status: CappedUsageProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_usage_profile("code-507", "label-726");
        assert_eq!(item.status, CappedUsageProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_capped_usage_profile("code-507", "label-726");
        if item.move_to(CappedUsageProfileStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedUsageProfileStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_usage_profile("code-507", "label-726");
        item.status = CappedUsageProfileStatus::Archived;
        assert!(
            item.move_to(CappedUsageProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
