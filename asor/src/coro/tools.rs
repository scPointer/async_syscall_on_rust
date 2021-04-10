pub struct Queue<T> {
    v: Vec<T>,
    tail: usize,
    head: usize,
    max_size: usize,
}

impl <T> Queue<T> {
    pub fn new(max_size:usize) -> Self {
        Queue {
            v: Vec::new(),
            tail: 0,
            head: 0,
            max_size: max_size
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tail == self.head
    }

    pub fn push(&mut self, task:T) -> Result<(), ()> {
        if self.v.len() < self.max_size {
            self.v.push(task);
            self.head += 1;
            Ok(())
        } else if self.head - self.tail < self.max_size {
            self.v[self.head % self.max_size] = task;
            self.head += 1;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn pop(&mut self) -> Result<&T, ()> {
        if self.tail < self.head {
            let pos = self.tail % self.max_size;
            self.tail += 1;
            let val = &self.v[pos];
            Ok(val)
        } else {
            Err(())
        }
    }
}