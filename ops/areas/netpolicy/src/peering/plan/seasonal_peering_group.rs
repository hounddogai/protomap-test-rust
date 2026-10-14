use super::*;

use acme_kit::text;

/// Where a seasonal peering group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalPeeringGroupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalPeeringGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SeasonalPeeringGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal peering group as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPeeringGroup {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub tags: Vec<String>,
    pub port: i64,
    pub channel: String,
    pub status: SeasonalPeeringGroupStatus,
}

impl SeasonalPeeringGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalPeeringGroupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalPeeringGroupStatus) -> bool {
        match self.status {
            SeasonalPeeringGroupStatus::Planned => {
                return next == SeasonalPeeringGroupStatus::Scheduled;
            }
            SeasonalPeeringGroupStatus::Scheduled => {
                match next {
                    SeasonalPeeringGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPeeringGroupStatus::InProgress => {
                match next {
                    SeasonalPeeringGroupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPeeringGroupStatus::Completed => {
                match next {
                    SeasonalPeeringGroupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SeasonalPeeringGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal peering group {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ends_at as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalPeeringGroup {
        SeasonalPeeringGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            tags: self.tags.clone(),
            port: self.port,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a seasonal peering group in status planned.
pub fn new_seasonal_peering_group(code: &str, title: &str) -> SeasonalPeeringGroup {
    SeasonalPeeringGroup {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        tags: Vec::new(),
        port: 0,
        channel: "".to_string(),
        status: SeasonalPeeringGroupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_seasonal_peering_group("code-954", "title-812");
        assert_eq!(item.status, SeasonalPeeringGroupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_seasonal_peering_group("code-954", "title-812");
        if item.move_to(SeasonalPeeringGroupStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalPeeringGroupStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_seasonal_peering_group("code-954", "title-812");
        item.status = SeasonalPeeringGroupStatus::Cancelled;
        assert!(
            item.move_to(SeasonalPeeringGroupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
