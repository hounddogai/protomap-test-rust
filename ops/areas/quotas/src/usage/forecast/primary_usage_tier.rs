use super::*;

use acme_kit::text;

/// Where a primary usage tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryUsageTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryUsageTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryUsageTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary usage tier as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageTier {
    pub code: String,
    pub title: String,
    pub label: String,
    pub cluster: String,
    pub name: String,
    pub summary: String,
    pub window_days: i64,
    pub channel: String,
    pub port: i64,
    pub status: PrimaryUsageTierStatus,
}

impl PrimaryUsageTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryUsageTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryUsageTierStatus) -> bool {
        match self.status {
            PrimaryUsageTierStatus::Requested => {
                match next {
                    PrimaryUsageTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryUsageTierStatus::Granted => {
                return next == PrimaryUsageTierStatus::Expired;
            }
            PrimaryUsageTierStatus::Revoked => {
                match next {
                    PrimaryUsageTierStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryUsageTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary usage tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary usage tier", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.port as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryUsageTier {
        PrimaryUsageTier {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            window_days: self.window_days,
            channel: self.channel.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary usage tier in status requested.
pub fn new_primary_usage_tier(code: &str, title: &str) -> PrimaryUsageTier {
    PrimaryUsageTier {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        cluster: "".to_string(),
        name: "".to_string(),
        summary: "".to_string(),
        window_days: 0,
        channel: "".to_string(),
        port: 0,
        status: PrimaryUsageTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_usage_tier("code-483", "title-489");
        assert_eq!(item.status, PrimaryUsageTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_primary_usage_tier("code-483", "title-489");
        if item.move_to(PrimaryUsageTierStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryUsageTierStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_usage_tier("code-483", "title-489");
        item.status = PrimaryUsageTierStatus::Expired;
        assert!(
            item.move_to(PrimaryUsageTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
