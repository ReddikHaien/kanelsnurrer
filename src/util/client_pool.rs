use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::{Mutex, MutexGuard},
};

pub struct ResourcePool<R> {
    instances: Pin<Box<[Mutex<(R, bool)>]>>,
}

impl<R> ResourcePool<R> {
    pub fn new<F>(creator: F, max_count: usize) -> Self
    where
        F: Fn() -> R,
    {
        let mut v = Vec::with_capacity(max_count);

        for _ in 0..max_count {
            v.push(Mutex::new((creator(), false)));
        }

        Self {
            instances: v.into_boxed_slice().into(),
        }
    }

    pub fn get_instance(&self) -> Option<ResourceGuard<'_, R>> {
        for x in self.instances.iter() {
            match x.try_lock() {
                Ok(data) => {
                    if !data.1 {
                        return Some(ResourceGuard { entry: data });
                    }
                }
                Err(_) => (),
            }
        }

        None
    }
}

pub struct ResourceGuard<'a, R> {
    entry: MutexGuard<'a, (R, bool)>,
}

impl<'a, R> Deref for ResourceGuard<'a, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.entry.0
    }
}

impl<'a, R> DerefMut for ResourceGuard<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entry.0
    }
}

impl<'a, R> Drop for ResourceGuard<'a, R> {
    fn drop(&mut self) {
        self.entry.1 = false;
    }
}
