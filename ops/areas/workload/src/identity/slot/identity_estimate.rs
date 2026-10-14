use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a identity estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityEstimateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IdentityEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IdentityEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEstimate {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub updated_at: i64,
    pub owner_id: String,
    pub score: f64,
    pub unit: String,
    pub attributes: BTreeMap<String, String>,
    pub name: String,
    pub reviewer: String,
    pub status: IdentityEstimateStatus,
}

impl IdentityEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityEstimateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityEstimateStatus) -> bool {
        match self.status {
            IdentityEstimateStatus::Created => {
                match next {
                    IdentityEstimateStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityEstimateStatus::Validated => {
                match next {
                    IdentityEstimateStatus::Published | IdentityEstimateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityEstimateStatus::Published => {
                match next {
                    IdentityEstimateStatus::Retired => {
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

    pub fn move_to(&mut self, next: IdentityEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity estimate {} cannot move from {} to {}",
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
        format!("{} {}", "identity estimate", self.code)
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
        ((self.score + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityEstimate {
        IdentityEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            updated_at: self.updated_at,
            owner_id: self.owner_id.clone(),
            score: self.score,
            unit: self.unit.clone(),
            attributes: self.attributes.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a identity estimate in status created.
pub fn new_identity_estimate(code: &str, label: &str) -> IdentityEstimate {
    IdentityEstimate {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        updated_at: 0,
        owner_id: "".to_string(),
        score: 0.0,
        unit: "".to_string(),
        attributes: BTreeMap::new(),
        name: "".to_string(),
        reviewer: "".to_string(),
        status: IdentityEstimateStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_identity_estimate("code-123", "label-677");
        assert_eq!(item.status, IdentityEstimateStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_identity_estimate("code-123", "label-677");
        if item.move_to(IdentityEstimateStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityEstimateStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_identity_estimate("code-123", "label-677");
        item.status = IdentityEstimateStatus::Retired;
        assert!(
            item.move_to(IdentityEstimateStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
