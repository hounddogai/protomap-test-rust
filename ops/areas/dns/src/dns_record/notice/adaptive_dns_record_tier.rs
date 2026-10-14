use super::*;

use acme_kit::text;

/// Where a adaptive dns record tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveDnsRecordTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveDnsRecordTierStatus {
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

impl std::fmt::Display for AdaptiveDnsRecordTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive dns record tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDnsRecordTier {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub version: i64,
    pub country: String,
    pub margin: f64,
    pub rotation_days: i64,
    pub tenant_id: String,
    pub status: AdaptiveDnsRecordTierStatus,
}

impl AdaptiveDnsRecordTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveDnsRecordTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveDnsRecordTierStatus) -> bool {
        match self.status {
            AdaptiveDnsRecordTierStatus::Open => {
                return next == AdaptiveDnsRecordTierStatus::Archived;
            }
            AdaptiveDnsRecordTierStatus::InReview => {
                match next {
                    AdaptiveDnsRecordTierStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDnsRecordTierStatus::Resolved => {
                match next {
                    AdaptiveDnsRecordTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDnsRecordTierStatus::Reopened => {
                match next {
                    AdaptiveDnsRecordTierStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveDnsRecordTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive dns record tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.rotation_days as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveDnsRecordTier {
        AdaptiveDnsRecordTier {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            version: self.version,
            country: self.country.clone(),
            margin: self.margin,
            rotation_days: self.rotation_days,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive dns record tier in status open.
pub fn new_adaptive_dns_record_tier(code: &str, title: &str) -> AdaptiveDnsRecordTier {
    AdaptiveDnsRecordTier {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        version: 0,
        country: "".to_string(),
        margin: 0.0,
        rotation_days: 0,
        tenant_id: "".to_string(),
        status: AdaptiveDnsRecordTierStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_adaptive_dns_record_tier("code-716", "title-81");
        assert_eq!(item.status, AdaptiveDnsRecordTierStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_adaptive_dns_record_tier("code-716", "title-81");
        if item.move_to(AdaptiveDnsRecordTierStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveDnsRecordTierStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_adaptive_dns_record_tier("code-716", "title-81");
        item.status = AdaptiveDnsRecordTierStatus::Archived;
        assert!(
            item.move_to(AdaptiveDnsRecordTierStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
