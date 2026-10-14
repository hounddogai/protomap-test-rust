use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a capped peering manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPeeringManifestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedPeeringManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CappedPeeringManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped peering manifest as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringManifest {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub amount: f64,
    pub cost: f64,
    pub score: f64,
    pub tenant_id: String,
    pub discount: f64,
    pub attributes: BTreeMap<String, String>,
    pub contact_email: String,
    pub updated_at: i64,
    pub status: CappedPeeringManifestStatus,
}

impl CappedPeeringManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPeeringManifestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPeeringManifestStatus) -> bool {
        match self.status {
            CappedPeeringManifestStatus::Draft => {
                return next == CappedPeeringManifestStatus::Suspended;
            }
            CappedPeeringManifestStatus::Pending => {
                return next == CappedPeeringManifestStatus::Active || next == CappedPeeringManifestStatus::Closed;
            }
            CappedPeeringManifestStatus::Active => {
                return next == CappedPeeringManifestStatus::Suspended;
            }
            CappedPeeringManifestStatus::Suspended => {
                return next == CappedPeeringManifestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedPeeringManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped peering manifest {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "capped peering manifest", self.code)
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.discount + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPeeringManifest {
        CappedPeeringManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            amount: self.amount,
            cost: self.cost,
            score: self.score,
            tenant_id: self.tenant_id.clone(),
            discount: self.discount,
            attributes: self.attributes.clone(),
            contact_email: self.contact_email.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped peering manifest in status draft.
pub fn new_capped_peering_manifest(code: &str, name: &str) -> CappedPeeringManifest {
    CappedPeeringManifest {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        amount: 0.0,
        cost: 0.0,
        score: 0.0,
        tenant_id: "".to_string(),
        discount: 0.0,
        attributes: BTreeMap::new(),
        contact_email: "".to_string(),
        updated_at: 0,
        status: CappedPeeringManifestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_peering_manifest("code-32", "name-906");
        assert_eq!(item.status, CappedPeeringManifestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_capped_peering_manifest("code-32", "name-906");
        if item.move_to(CappedPeeringManifestStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedPeeringManifestStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_peering_manifest("code-32", "name-906");
        item.status = CappedPeeringManifestStatus::Closed;
        assert!(
            item.move_to(CappedPeeringManifestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
