use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy probe preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyProbePreferenceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyProbePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyProbePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy probe preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbePreference {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub attributes: BTreeMap<String, String>,
    pub source: String,
    pub host: String,
    pub floor_value: f64,
    pub status: LegacyProbePreferenceStatus,
}

impl LegacyProbePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyProbePreferenceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyProbePreferenceStatus) -> bool {
        match self.status {
            LegacyProbePreferenceStatus::Requested => {
                match next {
                    LegacyProbePreferenceStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyProbePreferenceStatus::Granted => {
                match next {
                    LegacyProbePreferenceStatus::Revoked | LegacyProbePreferenceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyProbePreferenceStatus::Revoked => {
                return next == LegacyProbePreferenceStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyProbePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy probe preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> LegacyProbePreference {
        LegacyProbePreference {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            attributes: self.attributes.clone(),
            source: self.source.clone(),
            host: self.host.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a legacy probe preference in status requested.
pub fn new_legacy_probe_preference(code: &str, label: &str) -> LegacyProbePreference {
    LegacyProbePreference {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        attributes: BTreeMap::new(),
        source: "".to_string(),
        host: "".to_string(),
        floor_value: 0.0,
        status: LegacyProbePreferenceStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_probe_preference("code-251", "label-376");
        assert_eq!(item.status, LegacyProbePreferenceStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_legacy_probe_preference("code-251", "label-376");
        if item.move_to(LegacyProbePreferenceStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyProbePreferenceStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_probe_preference("code-251", "label-376");
        item.status = LegacyProbePreferenceStatus::Expired;
        assert!(
            item.move_to(LegacyProbePreferenceStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
