use std::cell::RefCell;

pub fn take_once<T>(cell: &RefCell<Option<T>>) -> T {
    cell.borrow_mut().take().expect("value initialized twice")
}
