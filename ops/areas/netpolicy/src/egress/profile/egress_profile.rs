use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a egress profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EgressProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EgressProfileStatus {
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

impl std::fmt::Display for EgressProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A egress profile as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressProfile {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub starts_at: i64,
    pub minimum: f64,
    pub owner_id: String,
    pub namespace: String,
    pub status: EgressProfileStatus,
}

impl EgressProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EgressProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EgressProfileStatus) -> bool {
        match self.status {
            EgressProfileStatus::Draft => {
                return next == EgressProfileStatus::Suspended;
            }
            EgressProfileStatus::Pending => {
                return next == EgressProfileStatus::Suspended || next == EgressProfileStatus::Closed;
            }
            EgressProfileStatus::Active => {
                return next == EgressProfileStatus::Suspended;
            }
            EgressProfileStatus::Suspended => {
                match next {
                    EgressProfileStatus::Closed => {
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

    pub fn move_to(&mut self, next: EgressProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "egress profile {} cannot move from {} to {}",
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
        format!("{} {}", "egress profile", self.code)
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

    pub fn net_exposure(&self) -> f64 {
        ((self.starts_at as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EgressProfile {
        EgressProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            starts_at: self.starts_at,
            minimum: self.minimum,
            owner_id: self.owner_id.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a egress profile in status draft.
pub fn new_egress_profile(code: &str, label: &str) -> EgressProfile {
    EgressProfile {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        starts_at: 0,
        minimum: 0.0,
        owner_id: "".to_string(),
        namespace: "".to_string(),
        status: EgressProfileStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_egress_profile("code-216", "label-656");
        assert_eq!(item.status, EgressProfileStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_egress_profile("code-216", "label-656");
        if item.move_to(EgressProfileStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EgressProfileStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_egress_profile("code-216", "label-656");
        item.status = EgressProfileStatus::Closed;
        assert!(
            item.move_to(EgressProfileStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
