use super::*;

use std::rc::Rc;

use acme_kit::clock;
use acme_kit::flags;

/// Moves strict action item drafts through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct StrictActionItemDraftService {
    pub store: Rc<dyn StrictActionItemDraftStore>,
    pub validator: StrictActionItemDraftValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
    pub engine: StrictActionItemDraftEngine,
}

impl StrictActionItemDraftService {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict action item draft {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(StrictActionItemDraftStatus::Warming)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is warming", code));
        Ok(())
    }

    pub fn close(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("strict action item draft {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(StrictActionItemDraftStatus::Stopped)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is stopped: {}", code, reason));
        Ok(())
    }

    pub fn evaluate(&mut self, code: &str) -> Result<f64, String> {
        let Some(item) = self.store.find(code) else {
            return Err(format!("strict action item draft {} not found", code));
        };
        let result = self.engine.compute(&item);
        self.note(&format!("{} evaluated at {}", code, result));
        Ok(result)
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            if item.updated_at > cutoff {
                continue;
            }
            let problems = self.validator.validate(&item);
            if (problems.len() as i64) == 0 {
                continue;
            }
            self.note(&format!("{} needs attention: {}", item.code, problems.join(", ")));
            changed += 1;
        }
        changed
    }
}

pub fn new_strict_action_item_draft_service(
    store: Rc<dyn StrictActionItemDraftStore>,
    clock: Rc<dyn clock::Clock>,
    flags: &flags::Registry,
) -> StrictActionItemDraftService {
    StrictActionItemDraftService {
        store: store.clone(),
        validator: default_strict_action_item_draft_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
        engine: default_strict_action_item_draft_engine(flags),
    }
}
