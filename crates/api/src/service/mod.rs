pub mod auth;
pub mod http;

use std::any::{
    type_name,
    Any,
    TypeId,
};

trait Service: 'static {}
pub trait ServiceLocator: 'static {
    fn locate_dyn(&mut self, id: TypeId) -> anyhow::Result<&mut dyn Any>;
}

impl dyn ServiceLocator {
    #[inline]
    #[allow(private_bounds)]
    pub fn locate<T: Service>(&mut self) -> &mut T {
        self.locate_dyn(TypeId::of::<T>())
            .unwrap_or_else(|e| panic!("service `{}` not configured: {e}", type_name::<T>()))
            .downcast_mut()
            .unwrap_or_else(|| panic!("couldn't cast service to `{}`", type_name::<T>()))
    }
}
