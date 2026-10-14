use super::*;

use acme_kit::text;

/// Where a adaptive probe entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveProbeEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveProbeEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveProbeEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive probe entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeEntry {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub attempts: i64,
    pub active: bool,
    pub burst: i64,
    pub country: String,
    pub discount: f64,
    pub tenant_id: String,
    pub status: AdaptiveProbeEntryStatus,
}

impl AdaptiveProbeEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveProbeEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveProbeEntryStatus) -> bool {
        match self.status {
            AdaptiveProbeEntryStatus::Proposed => {
                match next {
                    AdaptiveProbeEntryStatus::Approved | AdaptiveProbeEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveProbeEntryStatus::Approved => {
                match next {
                    AdaptiveProbeEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveProbeEntryStatus::Rejected => {
                return next == AdaptiveProbeEntryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveProbeEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive probe entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.discount * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveProbeEntry {
        AdaptiveProbeEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            attempts: self.attempts,
            active: self.active,
            burst: self.burst,
            country: self.country.clone(),
            discount: self.discount,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a adaptive probe entry in status proposed.
pub fn new_adaptive_probe_entry(code: &str, name: &str) -> AdaptiveProbeEntry {
    AdaptiveProbeEntry {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        attempts: 0,
        active: false,
        burst: 0,
        country: "".to_string(),
        discount: 0.0,
        tenant_id: "".to_string(),
        status: AdaptiveProbeEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_adaptive_probe_entry("code-258", "name-703");
        assert_eq!(item.status, AdaptiveProbeEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_adaptive_probe_entry("code-258", "name-703");
        if item.move_to(AdaptiveProbeEntryStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveProbeEntryStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_adaptive_probe_entry("code-258", "name-703");
        item.status = AdaptiveProbeEntryStatus::Withdrawn;
        assert!(
            item.move_to(AdaptiveProbeEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
