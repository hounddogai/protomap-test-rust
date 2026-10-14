use super::*;

use acme_kit::text;

/// Where a limit group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitGroupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LimitGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LimitGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitGroup {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub name: String,
    pub fingerprint: String,
    pub quota_units: i64,
    pub budget: f64,
    pub cipher: String,
    pub contact_email: String,
    pub notes: String,
    pub floor_value: f64,
    pub status: LimitGroupStatus,
}

impl LimitGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitGroupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitGroupStatus) -> bool {
        match self.status {
            LimitGroupStatus::Created => {
                match next {
                    LimitGroupStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitGroupStatus::Validated => {
                match next {
                    LimitGroupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitGroupStatus::Published => {
                match next {
                    LimitGroupStatus::Retired => {
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

    pub fn move_to(&mut self, next: LimitGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quota_units as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitGroup {
        LimitGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            quota_units: self.quota_units,
            budget: self.budget,
            cipher: self.cipher.clone(),
            contact_email: self.contact_email.clone(),
            notes: self.notes.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a limit group in status created.
pub fn new_limit_group(code: &str, title: &str) -> LimitGroup {
    LimitGroup {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        name: "".to_string(),
        fingerprint: "".to_string(),
        quota_units: 0,
        budget: 0.0,
        cipher: "".to_string(),
        contact_email: "".to_string(),
        notes: "".to_string(),
        floor_value: 0.0,
        status: LimitGroupStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_limit_group("code-101", "title-316");
        assert_eq!(item.status, LimitGroupStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_limit_group("code-101", "title-316");
        if item.move_to(LimitGroupStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitGroupStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_limit_group("code-101", "title-316");
        item.status = LimitGroupStatus::Retired;
        assert!(
            item.move_to(LimitGroupStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
