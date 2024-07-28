//! Calculate the offset of between slices.
use crate::SliceLexer;

/// Calculate the offset of between slices.
pub trait Offset<Start = Self> {
    /// Offset between the first byte of `start` and the first byte of `self`.
    ///
    /// > **Note**: This is an offset, not an index, and may point to the end of input (`start.len()`) when `self` is exhausted.
    fn offset_from(&self, start: &Start) -> usize;
}

impl<'a> Offset<&'a [u8]> for SliceLexer<'a> {
    fn offset_from(&self, start: &&'a [u8]) -> usize {
        let start = (*start).as_ptr();
        let end = self.slice.as_ptr();

        end as usize - start as usize
    }
}

#[cfg(test)]
mod test {
    use crate::{num::LexWrite, str::LexAlloc, token::Lex, SliceLexer, Token};

    use super::Offset;

    #[test]
    fn plain() {
        let input = b"hello world" as &[u8];
        let lexer = SliceLexer::new(&input[5..]);

        let offset = lexer.offset_from(&input);
        assert_eq!(offset, 5);
    }

    #[test]
    fn exhausted() {
        let input = b"\"hello world\"" as &[u8];
        let mut lexer = SliceLexer::new(input);
        assert_eq!(lexer.ws_token(), Some(Token::Quote));

        let value = lexer.str_string().expect("should be valid string");
        assert_eq!(value, "hello world");

        let offset = lexer.offset_from(&input);
        assert_eq!(offset, input.len());
    }
}
