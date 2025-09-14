use std::array;
use std::fmt;
use std::mem;

pub struct Queue<T, const N: usize>
where
    T: Default + Sized + Copy,
{
    data: [T; N],
    start: usize,
    size: usize,
}

impl<T, const N: usize> fmt::Debug for Queue<T, N>
where
    T: Default + Sized + Copy + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Queue")
            .field("data", &self.data)
            .field("head", &self.start)
            .field("size", &self.size)
            .finish()
    }
}

impl<T, const N: usize> Queue<T, N>
where
    T: Default + Sized + Copy,
{
    pub fn new() -> Self {
        Self {
            data: array::from_fn(|_| Default::default()),
            start: 0,
            size: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.size == N {
            panic!("Queue max size - {N} is exceeded");
        }
        self.data[(self.start + self.size) % N] = value;
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        (self.size > 0).then(|| {
            let value = self.data[self.start];
            self.start += 1;
            if self.start == N {
                self.start = 0;
            }
            self.size -= 1;
            value
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Queue;

    #[test]
    fn test_queue() {
        let mut queue = Queue::<_, 3>::new();
        queue.push(1);
        queue.push(2);
        queue.push(3);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
        queue.push(1);
        queue.push(2);
        queue.push(3);
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
        queue.push(1);
        queue.push(2);
        queue.push(3);
        queue.pop();
        queue.pop();
        queue.push(2);
        queue.push(1);
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    #[should_panic]
    fn test_queue_panic() {
        let mut queue = Queue::<_, 0>::new();
        assert_eq!(queue.pop(), None);
        queue.push(1);
    }
}
