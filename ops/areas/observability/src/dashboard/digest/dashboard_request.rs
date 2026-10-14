use super::*;

use acme_kit::text;

/// Where a dashboard request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DashboardRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DashboardRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard request as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRequest {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub contact_email: String,
    pub rate: f64,
    pub status: DashboardRequestStatus,
}

impl DashboardRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardRequestStatus) -> bool {
        match self.status {
            DashboardRequestStatus::Requested => {
                match next {
                    DashboardRequestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardRequestStatus::Granted => {
                match next {
                    DashboardRequestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardRequestStatus::Revoked => {
                match next {
                    DashboardRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: DashboardRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> DashboardRequest {
        DashboardRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            contact_email: self.contact_email.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dashboard request in status requested.
pub fn new_dashboard_request(code: &str, title: &str) -> DashboardRequest {
    DashboardRequest {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        contact_email: "".to_string(),
        rate: 0.0,
        status: DashboardRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_dashboard_request("code-472", "title-91");
        assert_eq!(item.status, DashboardRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_dashboard_request("code-472", "title-91");
        if item.move_to(DashboardRequestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardRequestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_dashboard_request("code-472", "title-91");
        item.status = DashboardRequestStatus::Expired;
        assert!(
            item.move_to(DashboardRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
