use std::{iter::Peekable, str::Chars};

/// Trying to peek two characters ahead with Peekable
/// turned out to be more difficult than I likes
/// This class buffers at most of characters to allow peek_two
pub struct Source<'a> {
    characters: Peekable<Chars<'a>>,
    buffered: Option<char>,
}

impl<'a> Source<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            characters: source.chars().peekable(),
            buffered: None,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<char> {
        if let Some(buffered) = self.buffered.take() {
            Some(buffered)
        } else {
            self.characters.next()
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        if let Some(buffered) = self.buffered.as_ref() {
            Some(*buffered)
        } else {
            self.characters.peek().copied()
        }
    }

    pub fn peek_two(&mut self) -> Option<char> {
        // If we have buffered character, we already popped off the peek
        if self.buffered.is_some() {
            self.characters.peek().copied()
        } else {
            self.buffered = self.characters.next();
            self.characters.peek().copied()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Source;

    #[test]
    fn walks() {
        let input = "asdf".to_string();
        let mut source = Source::new(&input);
        assert_eq!(
            [Some('a'), Some('s'), Some('d'), Some('f'), None],
            *(0..5).map(|_| source.next()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn peeks() {
        let input = "asdf".to_string();
        let mut source = Source::new(&input);
        assert_eq!(Some('a'), source.peek());
        assert_eq!(Some('a'), source.peek());
        assert_eq!(Some('a'), source.next());

        assert_eq!(Some('s'), source.peek());
        assert_eq!(Some('s'), source.peek());
        assert_eq!(Some('s'), source.next());

        assert_eq!(Some('d'), source.peek());
        assert_eq!(Some('d'), source.peek());
        assert_eq!(Some('d'), source.next());

        assert_eq!(Some('f'), source.peek());
        assert_eq!(Some('f'), source.peek());
        assert_eq!(Some('f'), source.next());

        assert_eq!(None, source.peek());
        assert_eq!(None, source.peek());
        assert_eq!(None, source.next());
    }

    #[test]
    fn peeks_two() {
        let input = "asdf".to_string();
        let mut source = Source::new(&input);
        assert_eq!(Some('a'), source.peek());
        assert_eq!(Some('s'), source.peek_two());
        assert_eq!(Some('s'), source.peek_two());
        assert_eq!(Some('a'), source.peek());
        assert_eq!(Some('a'), source.next());

        assert_eq!(Some('s'), source.peek());
        assert_eq!(Some('d'), source.peek_two());
        assert_eq!(Some('d'), source.peek_two());
        assert_eq!(Some('s'), source.peek());
        assert_eq!(Some('s'), source.next());

        assert_eq!(Some('d'), source.peek());
        assert_eq!(Some('f'), source.peek_two());
        assert_eq!(Some('f'), source.peek_two());
        assert_eq!(Some('d'), source.peek());
        assert_eq!(Some('d'), source.next());

        assert_eq!(Some('f'), source.peek());
        assert_eq!(None, source.peek_two());
        assert_eq!(None, source.peek_two());
        assert_eq!(Some('f'), source.peek());
        assert_eq!(Some('f'), source.next());

        assert_eq!(None, source.peek());
        assert_eq!(None, source.peek_two());
        assert_eq!(None, source.peek_two());
        assert_eq!(None, source.peek());
        assert_eq!(None, source.next());
    }
}
