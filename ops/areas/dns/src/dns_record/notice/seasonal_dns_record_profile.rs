use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal dns record profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDnsRecordProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalDnsRecordProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalDnsRecordProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal dns record profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordProfile {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub discount: f64,
    pub notes: String,
    pub principal: String,
    pub category: String,
    pub host: String,
    pub attributes: BTreeMap<String, String>,
    pub active: bool,
    pub threshold: f64,
    pub status: SeasonalDnsRecordProfileStatus,
}

impl SeasonalDnsRecordProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDnsRecordProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDnsRecordProfileStatus) -> bool {
        match self.status {
            SeasonalDnsRecordProfileStatus::Requested => {
                return next == SeasonalDnsRecordProfileStatus::Revoked
                    || next == SeasonalDnsRecordProfileStatus::Expired;
            }
            SeasonalDnsRecordProfileStatus::Granted => {
                match next {
                    SeasonalDnsRecordProfileStatus::Revoked | SeasonalDnsRecordProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDnsRecordProfileStatus::Revoked => {
                match next {
                    SeasonalDnsRecordProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalDnsRecordProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal dns record profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal dns record profile", self.code)
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
        (self.minimum * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDnsRecordProfile {
        SeasonalDnsRecordProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            discount: self.discount,
            notes: self.notes.clone(),
            principal: self.principal.clone(),
            category: self.category.clone(),
            host: self.host.clone(),
            attributes: self.attributes.clone(),
            active: self.active,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a seasonal dns record profile in status requested.
pub fn new_seasonal_dns_record_profile(code: &str, name: &str) -> SeasonalDnsRecordProfile {
    SeasonalDnsRecordProfile {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        discount: 0.0,
        notes: "".to_string(),
        principal: "".to_string(),
        category: "".to_string(),
        host: "".to_string(),
        attributes: BTreeMap::new(),
        active: false,
        threshold: 0.0,
        status: SeasonalDnsRecordProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_dns_record_profile("code-544", "name-647");
        assert_eq!(item.status, SeasonalDnsRecordProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_seasonal_dns_record_profile("code-544", "name-647");
        if item.move_to(SeasonalDnsRecordProfileStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalDnsRecordProfileStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_dns_record_profile("code-544", "name-647");
        item.status = SeasonalDnsRecordProfileStatus::Expired;
        assert!(
            item.move_to(SeasonalDnsRecordProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
