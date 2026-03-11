use spacetimedb::{Identity, ReducerContext, rand::Rng};

const MAGIC_HEADER: [u8; 3] = [0x49, 0x4b, 0x41];

pub trait IdentityListExt {
    fn display_identities(&self) -> String;
}

impl IdentityListExt for [Identity] {
    fn display_identities(&self) -> String {
        let inner = self
            .iter()
            .map(|user_id| format!("0x{user_id}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{inner}]")
    }
}

pub trait IdentityExt {
    fn is_generated(&self) -> bool;
}

impl IdentityExt for Identity {
    fn is_generated(&self) -> bool {
        let bytes = self.to_be_byte_array();
        bytes[0] == MAGIC_HEADER[0] && bytes[1] == MAGIC_HEADER[1] && bytes[2] == MAGIC_HEADER[2]
    }
}

pub trait ReducerContextExt {
    fn random_identity(&self) -> Identity;
    fn one_in(&self, chance: u64) -> bool;
    fn weighted_choice<'a, T>(&self, items: &'a [(u64, T)]) -> Option<&'a T>;
    fn random_percent(&self) -> f64;
    fn random_index(&self, length: usize) -> Option<usize>;
    fn pick_random<'a, T>(&self, values: &'a [T]) -> Option<&'a T>;
}

impl ReducerContextExt for ReducerContext {
    fn random_identity(&self) -> Identity {
        let mut bytes = [0u8; 32];
        bytes[0] = MAGIC_HEADER[0];
        bytes[1] = MAGIC_HEADER[1];
        bytes[2] = MAGIC_HEADER[2];
        for byte in bytes[3..].iter_mut() {
            *byte = self.random();
        }
        Identity::from_be_byte_array(bytes)
    }

    fn one_in(&self, chance: u64) -> bool {
        if chance == 0 {
            return false;
        }
        self.random::<u64>().is_multiple_of(chance)
    }

    fn weighted_choice<'a, T>(&self, items: &'a [(u64, T)]) -> Option<&'a T> {
        for (chance, item) in items.iter().rev() {
            if self.one_in(*chance) {
                return Some(item);
            }
        }
        None
    }

    fn random_percent(&self) -> f64 {
        self.rng().gen_range(0.0..=1.0)
    }

    fn random_index(&self, length: usize) -> Option<usize> {
        if length == 0 {
            return None;
        }
        Some(self.rng().gen_range(0..length))
    }

    fn pick_random<'a, T>(&self, values: &'a [T]) -> Option<&'a T> {
        let index = self.random_index(values.len())?;
        values.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spacetimedb::Identity;

    #[test]
    fn generated_identity_prefix_is_detected() {
        let mut bytes = [0u8; 32];
        bytes[0] = 0x49;
        bytes[1] = 0x4b;
        bytes[2] = 0x41;
        let generated = Identity::from_be_byte_array(bytes);
        assert!(generated.is_generated());
    }
}
