mod machine;
mod special_characters;
mod word;

// Re-export symbols.
pub use machine::German;
pub(self) use special_characters::{Casing, SpecialCharacter, Umlaut};
pub(self) use word::Word;
