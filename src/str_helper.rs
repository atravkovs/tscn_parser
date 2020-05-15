pub trait StrHelper {
    fn check_borders(&self, start_char: char, end_char: char) -> bool;
}

impl StrHelper for str {
    #[inline]
    fn check_borders(&self, start_char: char, end_char: char) -> bool {
        let mut chars = self.chars();

        let first = chars.nth(0);
        let last = chars.rev().nth(0);

        if first.is_none() || last.is_none() {
            return false;
        }

        first.unwrap() == start_char && last.unwrap() == end_char
    }
}
