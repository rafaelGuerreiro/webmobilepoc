use crate::{
    constants::{
        CHARACTER_NAME_MAX_LEN, CHARACTER_NAME_MIN_LEN, DEFAULT_CHARACTER_ATTACK_SPEED, DEFAULT_CHARACTER_CAPACITY,
        DEFAULT_CHARACTER_EXPERIENCE, DEFAULT_CHARACTER_HEALTH, DEFAULT_CHARACTER_LEVEL, DEFAULT_CHARACTER_MANA,
        DEFAULT_CHARACTER_SPEED,
    },
    error::{ErrorMapper, ResultExt, ServiceError, ServiceResult},
    extend::validate::ReducerContextRequirements,
    repository::{
        character::{
            CharacterStatsV1, CharacterV1, OnlineCharacterV1, character_stats_v1, character_v1, online_character_v1,
            types::{ClassV1, GenderV1, RaceV1},
        },
        event::services::EventReducerContext,
        item::services::ItemReducerContext,
    },
};
use spacetimedb::{Identity, ReducerContext, Table};
use std::ops::Deref;
use thiserror::Error;

pub trait CharacterReducerContext {
    fn character_services(&self) -> CharacterServices<'_>;
}

impl CharacterReducerContext for ReducerContext {
    fn character_services(&self) -> CharacterServices<'_> {
        CharacterServices { ctx: self }
    }
}

pub struct CharacterServices<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for CharacterServices<'_> {
    type Target = ReducerContext;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl CharacterServices<'_> {
    /// Finds a character by ID even if it's offline (not currently selected by the user).
    pub fn find_offline(&self, character_id: u64) -> Option<CharacterV1> {
        self.db.character_v1().character_id().find(character_id)
    }

    /// Finds a character by ID and checks if it's online (currently selected by the user).
    pub fn find_online(&self, character_id: u64) -> Option<CharacterV1> {
        let character = self.find_offline(character_id)?;
        let current = self.db.online_character_v1().user_id().find(character.user_id)?;

        if current.character_id == character_id {
            Some(character)
        } else {
            None
        }
    }

    /// Finds the current online character for a user, if one is selected.
    pub fn find_current(&self, user_id: Identity) -> Option<CharacterV1> {
        let current = self.db.online_character_v1().user_id().find(user_id)?;
        self.find_offline(current.character_id)
    }

    pub fn find_stats(&self, character_id: u64) -> Option<CharacterStatsV1> {
        self.db.character_stats_v1().character_id().find(character_id)
    }

    /// Gets a character by ID, ensuring it exists regardless of selection status.
    pub fn get_offline(&self, character_id: u64) -> ServiceResult<CharacterV1> {
        self.find_offline(character_id)
            .ok_or_else(|| CharacterError::character_not_found(character_id))
    }

    pub fn get_online(&self, character_id: u64) -> ServiceResult<CharacterV1> {
        self.find_online(character_id)
            .ok_or_else(|| CharacterError::character_not_found(character_id))
    }

    pub fn get_current(&self, user_id: Identity) -> ServiceResult<CharacterV1> {
        self.find_current(user_id)
            .ok_or_else(|| CharacterError::character_not_selected(user_id))
    }

    pub fn get_stats(&self, character_id: u64) -> ServiceResult<CharacterStatsV1> {
        self.find_stats(character_id)
            .ok_or_else(|| CharacterError::character_not_found(character_id))
    }

    pub fn create_character(
        &self,
        user_id: Identity,
        display_name: String,
        gender: GenderV1,
        race: RaceV1,
    ) -> ServiceResult<()> {
        let (display_name, canonical_name) = self.prepare_character_names(display_name)?;

        let character = self.db.character_v1().try_insert(CharacterV1 {
            character_id: 0,
            user_id,
            name: canonical_name,
            display_name: display_name.clone(),
            class: ClassV1::None,
            race,
            gender,
            created_at: self.timestamp,
        });

        let character = match character {
            Ok(character) => character,
            Err(_err) => {
                return Err(CharacterError::name_taken(display_name));
            },
        };

        self.db
            .character_stats_v1()
            .try_insert(CharacterStatsV1::new(&character))
            .map_conflict()?;

        self.ctx.item_services().create_starting_inventory(character.character_id)?;

        self.publish().character_created(user_id, character.character_id)?;
        self.select_character(user_id, character.character_id)?;

        Ok(())
    }

    pub fn select_character(&self, user_id: Identity, character_id: u64) -> ServiceResult<()> {
        let character = self.get_offline(character_id)?;
        if character.user_id != user_id {
            return Err(CharacterError::character_ownership_mismatch(character_id, user_id));
        }

        self.db.online_character_v1().user_id().insert_or_update(OnlineCharacterV1 {
            user_id,
            character_id,
            signed_in_at: self.timestamp,
        });

        self.publish().character_selected(user_id, character_id)?;
        Ok(())
    }

    pub fn unselect_character(&self, user_id: Identity) -> ServiceResult<()> {
        self.get_current(user_id)?;
        self.publish().character_unselected(user_id)?;
        Ok(())
    }

    pub fn clear_online_character(&self, user_id: Identity) {
        self.db.online_character_v1().user_id().delete(user_id);
    }

    fn prepare_character_names(&self, display_name: String) -> ServiceResult<(String, String)> {
        self.validate_str(
            &display_name,
            "Name",
            CHARACTER_NAME_MIN_LEN as u64,
            CHARACTER_NAME_MAX_LEN as u64,
        )?;

        let mut display_name = display_name.trim().to_string();
        while display_name.contains("  ") {
            display_name = display_name.replace("  ", " ");
        }

        let is_separator = |c: char| c == ' ' || c == '-' || c == '\'';

        if display_name
            .chars()
            .any(|character| !character.is_ascii_alphabetic() && !is_separator(character))
        {
            return Err(CharacterError::name_invalid_characters());
        }

        if display_name
            .chars()
            .zip(display_name.chars().skip(1))
            .any(|(a, b)| is_separator(a) && is_separator(b))
        {
            return Err(CharacterError::name_consecutive_separators());
        }

        if display_name.starts_with(|c: char| is_separator(c)) || display_name.ends_with(|c: char| is_separator(c)) {
            return Err(CharacterError::name_invalid_characters());
        }

        let canonical_name: String = display_name.chars().map(|character| character.to_ascii_lowercase()).collect();

        if canonical_name.is_empty() {
            return Err(CharacterError::name_without_letters());
        }

        self.validate_str(
            &canonical_name,
            "Name",
            CHARACTER_NAME_MIN_LEN as u64,
            CHARACTER_NAME_MAX_LEN as u64,
        )?;

        Ok((display_name, canonical_name))
    }
}

impl CharacterStatsV1 {
    pub fn new(character: &CharacterV1) -> Self {
        Self {
            character_id: character.character_id,
            user_id: character.user_id,
            level: DEFAULT_CHARACTER_LEVEL,
            experience: DEFAULT_CHARACTER_EXPERIENCE,
            health: DEFAULT_CHARACTER_HEALTH,
            mana: DEFAULT_CHARACTER_MANA,
            capacity: DEFAULT_CHARACTER_CAPACITY,
            free_capacity: DEFAULT_CHARACTER_CAPACITY,
            speed: DEFAULT_CHARACTER_SPEED,
            attack_speed: DEFAULT_CHARACTER_ATTACK_SPEED,
        }
    }
}

#[derive(Debug, Error)]
enum CharacterError {
    #[error("No current character selected for user {0}")]
    CharacterNotSelected(Identity),

    #[error("Character {0} was not found")]
    CharacterNotFound(u64),

    #[error("Character {character_id} does not belong to user {user_id}")]
    CharacterOwnershipMismatch { character_id: u64, user_id: Identity },

    #[error("Character name '{0}' is already taken")]
    NameTaken(String),

    #[error("Character name must contain only letters, spaces, hyphens, and apostrophes")]
    NameInvalidCharacters,

    #[error("Character name cannot have consecutive spaces, hyphens, or apostrophes")]
    NameConsecutiveSeparators,

    #[error("Character name must contain at least one letter")]
    NameWithoutLetters,
}

impl CharacterError {
    fn character_not_selected(user_id: Identity) -> ServiceError {
        Self::CharacterNotSelected(user_id).map_forbidden_error()
    }

    fn character_not_found(character_id: u64) -> ServiceError {
        Self::CharacterNotFound(character_id).map_not_found_error()
    }

    fn character_ownership_mismatch(character_id: u64, user_id: Identity) -> ServiceError {
        Self::CharacterOwnershipMismatch { character_id, user_id }.map_forbidden_error()
    }

    fn name_taken(display_name: String) -> ServiceError {
        Self::NameTaken(display_name).map_conflict_error()
    }

    fn name_invalid_characters() -> ServiceError {
        Self::NameInvalidCharacters.map_validation_error()
    }

    fn name_consecutive_separators() -> ServiceError {
        Self::NameConsecutiveSeparators.map_validation_error()
    }

    fn name_without_letters() -> ServiceError {
        Self::NameWithoutLetters.map_validation_error()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spacetimedb::ReducerContext;

    #[test]
    fn prepare_character_names_returns_display_and_canonical() {
        let dummy = ReducerContext::__dummy();
        let services = CharacterServices { ctx: &dummy };

        assert_eq!(
            services.prepare_character_names("  Sir     Galahad  ".to_string()).ok(),
            Some(("Sir Galahad".to_string(), "sir galahad".to_string()))
        );
    }

    #[test]
    fn prepare_character_names_keeps_spaces_in_canonical() {
        let dummy = ReducerContext::__dummy();
        let services = CharacterServices { ctx: &dummy };

        let (_, canonical_with_space) = services.prepare_character_names("Assas Sin".to_string()).unwrap();
        let (_, canonical_no_space) = services.prepare_character_names("Assassin".to_string()).unwrap();

        assert_eq!(canonical_with_space, "assas sin");
        assert_eq!(canonical_no_space, "assassin");
        assert_ne!(canonical_with_space, canonical_no_space);
    }

    #[test]
    fn prepare_character_names_allows_hyphens_and_apostrophes() {
        let dummy = ReducerContext::__dummy();
        let services = CharacterServices { ctx: &dummy };

        assert_eq!(
            services.prepare_character_names("O'Brien".to_string()).ok(),
            Some(("O'Brien".to_string(), "o'brien".to_string()))
        );
        assert_eq!(
            services.prepare_character_names("Dark-Knight".to_string()).ok(),
            Some(("Dark-Knight".to_string(), "dark-knight".to_string()))
        );
        assert_eq!(
            services.prepare_character_names("Good ol'day-dreamer".to_string()).ok(),
            Some(("Good ol'day-dreamer".to_string(), "good ol'day-dreamer".to_string()))
        );
        assert_eq!(
            services.prepare_character_names("Jean-Luc O'Neill".to_string()).ok(),
            Some(("Jean-Luc O'Neill".to_string(), "jean-luc o'neill".to_string()))
        );
        assert_eq!(
            services.prepare_character_names("Ana-Maria".to_string()).ok(),
            Some(("Ana-Maria".to_string(), "ana-maria".to_string()))
        );
    }

    #[test]
    fn prepare_character_names_rejects_non_letter_characters() {
        let dummy = ReducerContext::__dummy();
        let services = CharacterServices { ctx: &dummy };

        assert!(services.prepare_character_names("Name123".to_string()).is_err());
    }

    #[test]
    fn prepare_character_names_rejects_small_after_trimmed() {
        let dummy = ReducerContext::__dummy();
        let services = CharacterServices { ctx: &dummy };

        assert!(services.prepare_character_names(" ab ".to_string()).is_err());
        // "a bc" canonical is now "a bc" (4 chars), which meets the minimum length
        assert!(services.prepare_character_names("a bc".to_string()).is_ok());
    }

    #[test]
    fn prepare_character_names_rejects_space_only_input() {
        let dummy = ReducerContext::__dummy();
        let services = CharacterServices { ctx: &dummy };

        assert!(services.prepare_character_names("   ".to_string()).is_err());
    }

    #[test]
    fn prepare_character_names_rejects_consecutive_separators() {
        let dummy = ReducerContext::__dummy();
        let services = CharacterServices { ctx: &dummy };

        assert!(services.prepare_character_names("Dark--Knight".to_string()).is_err());
        assert!(services.prepare_character_names("Dark -Knight".to_string()).is_err());
        assert!(services.prepare_character_names("O''Brien".to_string()).is_err());
        assert!(services.prepare_character_names("Dark- Knight".to_string()).is_err());
    }

    #[test]
    fn prepare_character_names_rejects_leading_or_trailing_separators() {
        let dummy = ReducerContext::__dummy();
        let services = CharacterServices { ctx: &dummy };

        assert!(services.prepare_character_names("-Knight".to_string()).is_err());
        assert!(services.prepare_character_names("Knight-".to_string()).is_err());
        assert!(services.prepare_character_names("'Knight".to_string()).is_err());
        assert!(services.prepare_character_names("---".to_string()).is_err());
    }
}
