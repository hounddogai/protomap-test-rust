use super::*;

use acme_kit::text;

/// Where a regional suspension policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSuspensionPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalSuspensionPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalSuspensionPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional suspension policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSuspensionPolicy {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub minimum: f64,
    pub expires_at: i64,
    pub batch_size: i64,
    pub namespace: String,
    pub sequence: i64,
    pub discount: f64,
    pub host: String,
    pub status: RegionalSuspensionPolicyStatus,
}

impl RegionalSuspensionPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSuspensionPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSuspensionPolicyStatus) -> bool {
        match self.status {
            RegionalSuspensionPolicyStatus::Proposed => {
                return next == RegionalSuspensionPolicyStatus::Rejected
                    || next == RegionalSuspensionPolicyStatus::Withdrawn;
            }
            RegionalSuspensionPolicyStatus::Approved => {
                match next {
                    RegionalSuspensionPolicyStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSuspensionPolicyStatus::Rejected => {
                match next {
                    RegionalSuspensionPolicyStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalSuspensionPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional suspension policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.discount + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSuspensionPolicy {
        RegionalSuspensionPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            minimum: self.minimum,
            expires_at: self.expires_at,
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            sequence: self.sequence,
            discount: self.discount,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a regional suspension policy in status proposed.
pub fn new_regional_suspension_policy(code: &str, title: &str) -> RegionalSuspensionPolicy {
    RegionalSuspensionPolicy {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        minimum: 0.0,
        expires_at: 0,
        batch_size: 0,
        namespace: "".to_string(),
        sequence: 0,
        discount: 0.0,
        host: "".to_string(),
        status: RegionalSuspensionPolicyStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_suspension_policy("code-711", "title-639");
        assert_eq!(item.status, RegionalSuspensionPolicyStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_regional_suspension_policy("code-711", "title-639");
        if item.move_to(RegionalSuspensionPolicyStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSuspensionPolicyStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_suspension_policy("code-711", "title-639");
        item.status = RegionalSuspensionPolicyStatus::Withdrawn;
        assert!(
            item.move_to(RegionalSuspensionPolicyStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
