use spacetimedb::SpacetimeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, SpacetimeType)]
pub enum GenderV1 {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, SpacetimeType)]
pub enum RaceV1 {
    Human,
    Elf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, SpacetimeType)]
pub enum ClassV1 {
    None,
    Warrior,
    Rogue,
    Wizard,
    Berserker,
    Knight,
    Hunter,
    Archer,
    Warlock,
    Druid,
}
