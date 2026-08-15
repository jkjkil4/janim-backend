use std::{cell::RefCell, ops::Deref, rc::Rc};

enum CacheState<T> {
    Outdated,
    Computing,
    Valid(Rc<T>),
}

pub(super) struct RecursiveCache<T> {
    state: RefCell<CacheState<T>>,
}

impl<T> RecursiveCache<T> {
    pub(super) fn new() -> Self {
        Self {
            state: RefCell::new(CacheState::Outdated),
        }
    }

    #[inline]
    pub(super) fn reset(&self) {
        *self.state.borrow_mut() = CacheState::Outdated;
    }

    pub(super) fn get_or_compute<E>(
        &self,
        compute: impl FnOnce() -> Result<T, E>,
        cycle_error: impl FnOnce() -> E,
    ) -> Result<Rc<T>, E> {
        match self.state.borrow().deref() {
            CacheState::Outdated => {}
            CacheState::Computing => {
                return Err(cycle_error());
            }
            CacheState::Valid(value) => {
                return Ok(value.clone());
            }
        }

        *self.state.borrow_mut() = CacheState::Computing;

        match compute() {
            Ok(value) => {
                let value = Rc::new(value);
                *self.state.borrow_mut() = CacheState::Valid(value.clone());
                Ok(value)
            }
            Err(err) => {
                *self.state.borrow_mut() = CacheState::Outdated;
                Err(err)
            }
        }
    }
}
