use super::*;

use acme_kit::text;

/// Where a tiered silence cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSilenceCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredSilenceCycleStatus {
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

impl std::fmt::Display for TieredSilenceCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered silence cycle as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceCycle {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub port: i64,
    pub region: String,
    pub external_ref: String,
    pub visible: bool,
    pub amount: f64,
    pub starts_at: i64,
    pub status: TieredSilenceCycleStatus,
}

impl TieredSilenceCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSilenceCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSilenceCycleStatus) -> bool {
        match self.status {
            TieredSilenceCycleStatus::Draft => {
                return next == TieredSilenceCycleStatus::Active;
            }
            TieredSilenceCycleStatus::Pending => {
                match next {
                    TieredSilenceCycleStatus::Suspended | TieredSilenceCycleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSilenceCycleStatus::Active => {
                return next == TieredSilenceCycleStatus::Suspended || next == TieredSilenceCycleStatus::Closed;
            }
            TieredSilenceCycleStatus::Suspended => {
                match next {
                    TieredSilenceCycleStatus::Closed => {
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

    pub fn move_to(&mut self, next: TieredSilenceCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered silence cycle {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered silence cycle", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.starts_at as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredSilenceCycle {
        TieredSilenceCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            port: self.port,
            region: self.region.clone(),
            external_ref: self.external_ref.clone(),
            visible: self.visible,
            amount: self.amount,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered silence cycle in status draft.
pub fn new_tiered_silence_cycle(code: &str, label: &str) -> TieredSilenceCycle {
    TieredSilenceCycle {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        port: 0,
        region: "".to_string(),
        external_ref: "".to_string(),
        visible: false,
        amount: 0.0,
        starts_at: 0,
        status: TieredSilenceCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_silence_cycle("code-706", "label-781");
        assert_eq!(item.status, TieredSilenceCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_tiered_silence_cycle("code-706", "label-781");
        if item.move_to(TieredSilenceCycleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredSilenceCycleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_silence_cycle("code-706", "label-781");
        item.status = TieredSilenceCycleStatus::Closed;
        assert!(
            item.move_to(TieredSilenceCycleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
