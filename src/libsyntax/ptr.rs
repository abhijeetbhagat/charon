use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Clone)] //this is necessary so that TType can be used in assert, compared, cloned
pub struct B<T>{
    ptr : Box<T>
}

//acts like a constructor
pub fn B<T>(value : T)->B<T>{
    B {ptr : Box::new(value)}
}

impl<T> Deref for B<T>{ //allows & to be used for B<T>
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T{
        &self.ptr
    }
}

impl<T> DerefMut for B<T>{

   fn deref_mut<'a>(&'a mut self) -> &'a mut T{
        &mut self.ptr
   }
}
