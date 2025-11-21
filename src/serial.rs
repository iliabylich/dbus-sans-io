use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub(crate) struct Serial {
    counter: Rc<RefCell<u32>>,
}

impl Serial {
    pub(crate) fn zero() -> Self {
        Self {
            counter: Rc::new(RefCell::new(1)),
        }
    }

    pub(crate) fn increment(&self) {
        let mut counter = self.counter.borrow_mut();
        *counter += 1;
    }

    pub(crate) fn get(&self) -> u32 {
        let counter = self.counter.borrow();
        *counter
    }

    pub(crate) fn increment_and_get(&self) -> u32 {
        self.increment();
        self.get()
    }
}
