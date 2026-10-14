use super::*;

use acme_kit::text;

/// Where a allowlist notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AllowlistNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AllowlistNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistNotice {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub created_at: i64,
    pub maximum: f64,
    pub checksum: String,
    pub segment: String,
    pub weight: f64,
    pub phone: String,
    pub status: AllowlistNoticeStatus,
}

impl AllowlistNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistNoticeStatus) -> bool {
        match self.status {
            AllowlistNoticeStatus::Created => {
                match next {
                    AllowlistNoticeStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistNoticeStatus::Validated => {
                match next {
                    AllowlistNoticeStatus::Published | AllowlistNoticeStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistNoticeStatus::Published => {
                match next {
                    AllowlistNoticeStatus::Retired => {
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

    pub fn move_to(&mut self, next: AllowlistNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.weight * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistNotice {
        AllowlistNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            created_at: self.created_at,
            maximum: self.maximum,
            checksum: self.checksum.clone(),
            segment: self.segment.clone(),
            weight: self.weight,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a allowlist notice in status created.
pub fn new_allowlist_notice(code: &str, name: &str) -> AllowlistNotice {
    AllowlistNotice {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        created_at: 0,
        maximum: 0.0,
        checksum: "".to_string(),
        segment: "".to_string(),
        weight: 0.0,
        phone: "".to_string(),
        status: AllowlistNoticeStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_allowlist_notice("code-435", "name-152");
        assert_eq!(item.status, AllowlistNoticeStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_allowlist_notice("code-435", "name-152");
        if item.move_to(AllowlistNoticeStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistNoticeStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_allowlist_notice("code-435", "name-152");
        item.status = AllowlistNoticeStatus::Retired;
        assert!(
            item.move_to(AllowlistNoticeStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
