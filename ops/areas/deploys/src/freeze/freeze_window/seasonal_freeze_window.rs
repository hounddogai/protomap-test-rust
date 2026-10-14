use super::*;

use acme_kit::text;

/// Where a seasonal freeze window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFreezeWindowStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalFreezeWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalFreezeWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal freeze window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeWindow {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub host: String,
    pub audience: String,
    pub rotation_days: i64,
    pub namespace: String,
    pub checksum: String,
    pub amount: f64,
    pub reviewer: String,
    pub status: SeasonalFreezeWindowStatus,
}

impl SeasonalFreezeWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFreezeWindowStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFreezeWindowStatus) -> bool {
        match self.status {
            SeasonalFreezeWindowStatus::Requested => {
                return next == SeasonalFreezeWindowStatus::Granted;
            }
            SeasonalFreezeWindowStatus::Granted => {
                match next {
                    SeasonalFreezeWindowStatus::Revoked | SeasonalFreezeWindowStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFreezeWindowStatus::Revoked => {
                match next {
                    SeasonalFreezeWindowStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalFreezeWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal freeze window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.rotation_days as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalFreezeWindow {
        SeasonalFreezeWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            host: self.host.clone(),
            audience: self.audience.clone(),
            rotation_days: self.rotation_days,
            namespace: self.namespace.clone(),
            checksum: self.checksum.clone(),
            amount: self.amount,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a seasonal freeze window in status requested.
pub fn new_seasonal_freeze_window(code: &str, name: &str) -> SeasonalFreezeWindow {
    SeasonalFreezeWindow {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        host: "".to_string(),
        audience: "".to_string(),
        rotation_days: 0,
        namespace: "".to_string(),
        checksum: "".to_string(),
        amount: 0.0,
        reviewer: "".to_string(),
        status: SeasonalFreezeWindowStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_freeze_window("code-100", "name-703");
        assert_eq!(item.status, SeasonalFreezeWindowStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_seasonal_freeze_window("code-100", "name-703");
        if item.move_to(SeasonalFreezeWindowStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalFreezeWindowStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_freeze_window("code-100", "name-703");
        item.status = SeasonalFreezeWindowStatus::Expired;
        assert!(
            item.move_to(SeasonalFreezeWindowStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
