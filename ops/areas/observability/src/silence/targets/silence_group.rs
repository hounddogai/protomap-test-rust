use super::*;

use acme_kit::text;

/// Where a silence group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SilenceGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SilenceGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceGroup {
    pub code: String,
    pub name: String,
    pub region: String,
    pub scope: String,
    pub cipher: String,
    pub floor_value: f64,
    pub status: SilenceGroupStatus,
}

impl SilenceGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceGroupStatus) -> bool {
        match self.status {
            SilenceGroupStatus::Proposed => {
                return next == SilenceGroupStatus::Withdrawn;
            }
            SilenceGroupStatus::Approved => {
                match next {
                    SilenceGroupStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceGroupStatus::Rejected => {
                match next {
                    SilenceGroupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SilenceGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence group", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> SilenceGroup {
        SilenceGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            scope: self.scope.clone(),
            cipher: self.cipher.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a silence group in status proposed.
pub fn new_silence_group(code: &str, name: &str) -> SilenceGroup {
    SilenceGroup {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        scope: "".to_string(),
        cipher: "".to_string(),
        floor_value: 0.0,
        status: SilenceGroupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_silence_group("code-898", "name-954");
        assert_eq!(item.status, SilenceGroupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_silence_group("code-898", "name-954");
        if item.move_to(SilenceGroupStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceGroupStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_silence_group("code-898", "name-954");
        item.status = SilenceGroupStatus::Withdrawn;
        assert!(
            item.move_to(SilenceGroupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
