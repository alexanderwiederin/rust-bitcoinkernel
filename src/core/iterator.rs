use std::marker::PhantomData;

pub trait IterableCollection: Copy {
    type Item<'a>
    where
        Self: 'a;

    fn len(&self) -> usize;
    fn get<'a>(self, index: usize) -> Option<Self::Item<'a>>
    where
        Self: 'a;
}

pub struct Iter<'a, C> {
    collection: C,
    current_index: usize,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, C> Iter<'a, C> {
    pub fn new(collection: C) -> Self {
        Self {
            collection,
            current_index: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'a, C: IterableCollection + 'a> Iterator for Iter<'a, C> {
    type Item = C::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current_index;
        self.current_index += 1;
        self.collection.get(index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.collection.len().saturating_sub(self.current_index);
        (remaining, Some(remaining))
    }
}

impl<'a, C: IterableCollection + 'a> ExactSizeIterator for Iter<'a, C> {
    fn len(&self) -> usize {
        self.collection.len().saturating_sub(self.current_index)
    }
}
