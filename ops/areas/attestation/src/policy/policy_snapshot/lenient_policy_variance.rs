use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient policy variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPolicyVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientPolicyVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientPolicyVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient policy variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyVariance {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub phone: String,
    pub summary: String,
    pub attributes: BTreeMap<String, String>,
    pub label: String,
    pub quota_units: i64,
    pub batch_size: i64,
    pub status: LenientPolicyVarianceStatus,
}

impl LenientPolicyVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPolicyVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPolicyVarianceStatus) -> bool {
        match self.status {
            LenientPolicyVarianceStatus::Requested => {
                match next {
                    LenientPolicyVarianceStatus::Revoked | LenientPolicyVarianceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPolicyVarianceStatus::Granted => {
                return next == LenientPolicyVarianceStatus::Revoked || next == LenientPolicyVarianceStatus::Expired;
            }
            LenientPolicyVarianceStatus::Revoked => {
                return next == LenientPolicyVarianceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPolicyVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient policy variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn projected_total(&self) -> f64 {
        ((self.batch_size as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPolicyVariance {
        LenientPolicyVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            phone: self.phone.clone(),
            summary: self.summary.clone(),
            attributes: self.attributes.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient policy variance in status requested.
pub fn new_lenient_policy_variance(code: &str, title: &str) -> LenientPolicyVariance {
    LenientPolicyVariance {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        phone: "".to_string(),
        summary: "".to_string(),
        attributes: BTreeMap::new(),
        label: "".to_string(),
        quota_units: 0,
        batch_size: 0,
        status: LenientPolicyVarianceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_policy_variance("code-632", "title-255");
        assert_eq!(item.status, LenientPolicyVarianceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_lenient_policy_variance("code-632", "title-255");
        if item.move_to(LenientPolicyVarianceStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPolicyVarianceStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_policy_variance("code-632", "title-255");
        item.status = LenientPolicyVarianceStatus::Expired;
        assert!(
            item.move_to(LenientPolicyVarianceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
