use super::*;

use acme_kit::text;

/// Where a delegation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DelegationOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DelegationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationOverride {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub summary: String,
    pub duration_minutes: i64,
    pub channel: String,
    pub status: DelegationOverrideStatus,
}

impl DelegationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationOverrideStatus) -> bool {
        match self.status {
            DelegationOverrideStatus::Created => {
                match next {
                    DelegationOverrideStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationOverrideStatus::Validated => {
                match next {
                    DelegationOverrideStatus::Published | DelegationOverrideStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationOverrideStatus::Published => {
                match next {
                    DelegationOverrideStatus::Retired => {
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

    pub fn move_to(&mut self, next: DelegationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationOverride {
        DelegationOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            summary: self.summary.clone(),
            duration_minutes: self.duration_minutes,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a delegation override in status created.
pub fn new_delegation_override(code: &str, label: &str) -> DelegationOverride {
    DelegationOverride {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        summary: "".to_string(),
        duration_minutes: 0,
        channel: "".to_string(),
        status: DelegationOverrideStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_delegation_override("code-596", "label-190");
        assert_eq!(item.status, DelegationOverrideStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_delegation_override("code-596", "label-190");
        if item.move_to(DelegationOverrideStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationOverrideStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_delegation_override("code-596", "label-190");
        item.status = DelegationOverrideStatus::Retired;
        assert!(
            item.move_to(DelegationOverrideStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
