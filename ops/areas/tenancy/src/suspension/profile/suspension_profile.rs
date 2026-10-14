use super::*;

use acme_kit::text;

/// Where a suspension profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SuspensionProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SuspensionProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a suspension profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionProfile {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub segment: String,
    pub label: String,
    pub external_ref: String,
    pub status: SuspensionProfileStatus,
}

impl SuspensionProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionProfileStatus) -> bool {
        match self.status {
            SuspensionProfileStatus::Requested => {
                match next {
                    SuspensionProfileStatus::Revoked | SuspensionProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionProfileStatus::Granted => {
                return next == SuspensionProfileStatus::Revoked || next == SuspensionProfileStatus::Expired;
            }
            SuspensionProfileStatus::Revoked => {
                match next {
                    SuspensionProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: SuspensionProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> SuspensionProfile {
        SuspensionProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            segment: self.segment.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a suspension profile in status requested.
pub fn new_suspension_profile(code: &str, title: &str) -> SuspensionProfile {
    SuspensionProfile {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        segment: "".to_string(),
        label: "".to_string(),
        external_ref: "".to_string(),
        status: SuspensionProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_suspension_profile("code-491", "title-382");
        assert_eq!(item.status, SuspensionProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_suspension_profile("code-491", "title-382");
        if item.move_to(SuspensionProfileStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionProfileStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_suspension_profile("code-491", "title-382");
        item.status = SuspensionProfileStatus::Expired;
        assert!(
            item.move_to(SuspensionProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
