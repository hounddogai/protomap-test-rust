use super::*;

use acme_kit::text;

/// Where a nonce notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl NonceNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for NonceNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce notice as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceNotice {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub name: String,
    pub ends_at: i64,
    pub status: NonceNoticeStatus,
}

impl NonceNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceNoticeStatus) -> bool {
        match self.status {
            NonceNoticeStatus::Created => {
                match next {
                    NonceNoticeStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceNoticeStatus::Validated => {
                return next == NonceNoticeStatus::Published || next == NonceNoticeStatus::Retired;
            }
            NonceNoticeStatus::Published => {
                match next {
                    NonceNoticeStatus::Retired => {
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

    pub fn move_to(&mut self, next: NonceNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> NonceNotice {
        NonceNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a nonce notice in status created.
pub fn new_nonce_notice(code: &str, label: &str) -> NonceNotice {
    NonceNotice {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        name: "".to_string(),
        ends_at: 0,
        status: NonceNoticeStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_nonce_notice("code-616", "label-774");
        assert_eq!(item.status, NonceNoticeStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_nonce_notice("code-616", "label-774");
        if item.move_to(NonceNoticeStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceNoticeStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_nonce_notice("code-616", "label-774");
        item.status = NonceNoticeStatus::Retired;
        assert!(
            item.move_to(NonceNoticeStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
