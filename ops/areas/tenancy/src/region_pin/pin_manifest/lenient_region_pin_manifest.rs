use super::*;

use acme_kit::text;

/// Where a lenient region pin manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRegionPinManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientRegionPinManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientRegionPinManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient region pin manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinManifest {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub priority: i64,
    pub cost: f64,
    pub attempts: i64,
    pub capacity: i64,
    pub phone: String,
    pub amount: f64,
    pub starts_at: i64,
    pub scope: String,
    pub status: LenientRegionPinManifestStatus,
}

impl LenientRegionPinManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRegionPinManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRegionPinManifestStatus) -> bool {
        match self.status {
            LenientRegionPinManifestStatus::Requested => {
                return next == LenientRegionPinManifestStatus::Revoked;
            }
            LenientRegionPinManifestStatus::Granted => {
                match next {
                    LenientRegionPinManifestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRegionPinManifestStatus::Revoked => {
                match next {
                    LenientRegionPinManifestStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientRegionPinManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient region pin manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient region pin manifest", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRegionPinManifest {
        LenientRegionPinManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            priority: self.priority,
            cost: self.cost,
            attempts: self.attempts,
            capacity: self.capacity,
            phone: self.phone.clone(),
            amount: self.amount,
            starts_at: self.starts_at,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient region pin manifest in status requested.
pub fn new_lenient_region_pin_manifest(code: &str, label: &str) -> LenientRegionPinManifest {
    LenientRegionPinManifest {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        priority: 0,
        cost: 0.0,
        attempts: 0,
        capacity: 0,
        phone: "".to_string(),
        amount: 0.0,
        starts_at: 0,
        scope: "".to_string(),
        status: LenientRegionPinManifestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_region_pin_manifest("code-810", "label-679");
        assert_eq!(item.status, LenientRegionPinManifestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_lenient_region_pin_manifest("code-810", "label-679");
        if item.move_to(LenientRegionPinManifestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRegionPinManifestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_region_pin_manifest("code-810", "label-679");
        item.status = LenientRegionPinManifestStatus::Expired;
        assert!(
            item.move_to(LenientRegionPinManifestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
