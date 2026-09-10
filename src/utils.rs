#[derive(Clone, Eq, PartialEq)]
pub struct BidirectionalIter<'a, T> {
    data: &'a [T],
    pos: usize,
}

impl<'a, T> std::ops::Deref for BidirectionalIter<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data[self.pos]
    }
}

impl<'a, T> BidirectionalIter<'a, T> {
    pub fn new(data: &'a [T]) -> Self {
        BidirectionalIter { data, pos: 0 }
    }

    pub fn with_pos(mut self, pos: usize) -> Self {
        if pos < self.data.len() {
            self.pos = pos;
        }
        self
    }

    pub fn next(&mut self) -> Option<&'a T> {
        if self.pos < self.data.len() - 1 {
            self.pos += 1;
            Some(&self.data[self.pos])
        } else {
            None
        }
    }

    pub fn prev(&mut self) -> Option<&'a T> {
        if self.pos > 0 {
            self.pos -= 1;
            Some(&self.data[self.pos])
        } else {
            None
        }
    }
}
