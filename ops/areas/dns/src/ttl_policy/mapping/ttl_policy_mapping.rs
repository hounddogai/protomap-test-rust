use super::*;

use acme_kit::text;

/// Where a ttl policy mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyMappingStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TtlPolicyMappingStatus {
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

impl std::fmt::Display for TtlPolicyMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy mapping as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMapping {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub segment: String,
    pub floor_value: f64,
    pub attempts: i64,
    pub description: String,
    pub rotation_days: i64,
    pub status: TtlPolicyMappingStatus,
}

impl TtlPolicyMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyMappingStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyMappingStatus) -> bool {
        match self.status {
            TtlPolicyMappingStatus::Draft => {
                match next {
                    TtlPolicyMappingStatus::Active | TtlPolicyMappingStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyMappingStatus::Pending => {
                match next {
                    TtlPolicyMappingStatus::Active | TtlPolicyMappingStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyMappingStatus::Active => {
                match next {
                    TtlPolicyMappingStatus::Suspended | TtlPolicyMappingStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyMappingStatus::Suspended => {
                match next {
                    TtlPolicyMappingStatus::Closed => {
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

    pub fn move_to(&mut self, next: TtlPolicyMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.attempts as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyMapping {
        TtlPolicyMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            segment: self.segment.clone(),
            floor_value: self.floor_value,
            attempts: self.attempts,
            description: self.description.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a ttl policy mapping in status draft.
pub fn new_ttl_policy_mapping(code: &str, name: &str) -> TtlPolicyMapping {
    TtlPolicyMapping {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        segment: "".to_string(),
        floor_value: 0.0,
        attempts: 0,
        description: "".to_string(),
        rotation_days: 0,
        status: TtlPolicyMappingStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_ttl_policy_mapping("code-443", "name-731");
        assert_eq!(item.status, TtlPolicyMappingStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_ttl_policy_mapping("code-443", "name-731");
        if item.move_to(TtlPolicyMappingStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyMappingStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_ttl_policy_mapping("code-443", "name-731");
        item.status = TtlPolicyMappingStatus::Closed;
        assert!(
            item.move_to(TtlPolicyMappingStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
