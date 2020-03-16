pub trait StrHelper {
    fn check_borders(&self, start_char: char, end_char: char) -> bool;
}

impl StrHelper for str {
    #[inline]
    fn check_borders(&self, start_char: char, end_char: char) -> bool {
        let mut chars = self.chars();
        chars.nth(0).unwrap() == start_char && chars.rev().nth(0).unwrap() == end_char
    }
}
