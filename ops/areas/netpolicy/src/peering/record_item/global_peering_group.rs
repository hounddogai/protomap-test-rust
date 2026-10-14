use super::*;

use acme_kit::text;

/// Where a global peering group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPeeringGroupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalPeeringGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for GlobalPeeringGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global peering group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringGroup {
    pub code: String,
    pub title: String,
    pub label: String,
    pub ends_at: i64,
    pub region: String,
    pub cluster: String,
    pub channel: String,
    pub locale: String,
    pub score: f64,
    pub host: String,
    pub status: GlobalPeeringGroupStatus,
}

impl GlobalPeeringGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPeeringGroupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPeeringGroupStatus) -> bool {
        match self.status {
            GlobalPeeringGroupStatus::Draft => {
                return next == GlobalPeeringGroupStatus::Active;
            }
            GlobalPeeringGroupStatus::Pending => {
                return next == GlobalPeeringGroupStatus::Active;
            }
            GlobalPeeringGroupStatus::Active => {
                match next {
                    GlobalPeeringGroupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPeeringGroupStatus::Suspended => {
                return next == GlobalPeeringGroupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPeeringGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global peering group {} cannot move from {} to {}",
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
        format!("{} {}", "global peering group", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ends_at as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPeeringGroup {
        GlobalPeeringGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            region: self.region.clone(),
            cluster: self.cluster.clone(),
            channel: self.channel.clone(),
            locale: self.locale.clone(),
            score: self.score,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a global peering group in status draft.
pub fn new_global_peering_group(code: &str, title: &str) -> GlobalPeeringGroup {
    GlobalPeeringGroup {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        ends_at: 0,
        region: "".to_string(),
        cluster: "".to_string(),
        channel: "".to_string(),
        locale: "".to_string(),
        score: 0.0,
        host: "".to_string(),
        status: GlobalPeeringGroupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_peering_group("code-527", "title-556");
        assert_eq!(item.status, GlobalPeeringGroupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_global_peering_group("code-527", "title-556");
        if item.move_to(GlobalPeeringGroupStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPeeringGroupStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_peering_group("code-527", "title-556");
        item.status = GlobalPeeringGroupStatus::Closed;
        assert!(
            item.move_to(GlobalPeeringGroupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
