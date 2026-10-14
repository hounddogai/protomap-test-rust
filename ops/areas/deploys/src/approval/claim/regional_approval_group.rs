use super::*;

use acme_kit::text;

/// Where a regional approval group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalApprovalGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalApprovalGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalApprovalGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional approval group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalGroup {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub fingerprint: String,
    pub budget: f64,
    pub duration_minutes: i64,
    pub archived: bool,
    pub status: RegionalApprovalGroupStatus,
}

impl RegionalApprovalGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalApprovalGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalApprovalGroupStatus) -> bool {
        match self.status {
            RegionalApprovalGroupStatus::Created => {
                match next {
                    RegionalApprovalGroupStatus::Published | RegionalApprovalGroupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalApprovalGroupStatus::Validated => {
                match next {
                    RegionalApprovalGroupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalApprovalGroupStatus::Published => {
                match next {
                    RegionalApprovalGroupStatus::Retired => {
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

    pub fn move_to(&mut self, next: RegionalApprovalGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional approval group {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.budget + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalApprovalGroup {
        RegionalApprovalGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            fingerprint: self.fingerprint.clone(),
            budget: self.budget,
            duration_minutes: self.duration_minutes,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional approval group in status created.
pub fn new_regional_approval_group(code: &str, title: &str) -> RegionalApprovalGroup {
    RegionalApprovalGroup {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        fingerprint: "".to_string(),
        budget: 0.0,
        duration_minutes: 0,
        archived: false,
        status: RegionalApprovalGroupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_regional_approval_group("code-79", "title-481");
        assert_eq!(item.status, RegionalApprovalGroupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_regional_approval_group("code-79", "title-481");
        if item.move_to(RegionalApprovalGroupStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalApprovalGroupStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_regional_approval_group("code-79", "title-481");
        item.status = RegionalApprovalGroupStatus::Retired;
        assert!(
            item.move_to(RegionalApprovalGroupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
