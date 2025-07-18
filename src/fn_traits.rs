use crate::{Callable, IexResult};

macro_rules! define_signature_trait {
    ($signature_trait:ident$(, $generic:ident)*) => {
        pub trait $signature_trait {
            $(type $generic;)*
            type Output;
            type Error;
        }

        impl<T, E $(, $generic)*> $signature_trait for fn($($generic),*) -> Result<T, E> {
            $(type $generic = $generic;)*
            type Output = T;
            type Error = E;
        }
    };
}

macro_rules! define_fn_trait {
    ($native_trait:ident => $trait:ident via $signature_trait:ident$(, $generic:ident)*) => {
        pub trait $trait<Sig: signatures::$signature_trait>:
            $native_trait($(Sig::$generic),*) -> IexResult<Self::Imp, Sig::Output, Sig::Error>
        {
            type Imp: Callable;
        }

        impl<Sig: signatures::$signature_trait, Func, Imp: Callable> $trait<Sig> for Func
        where
            Func: $native_trait($(Sig::$generic),*) -> IexResult<Imp, Sig::Output, Sig::Error>,
        {
            type Imp = Imp;
        }
    };
}

macro_rules! define_signature_traits_for_arities {
    () => {};
    ($head:tt$(, $tail:tt)*) => {
        define_signature_traits_for_arities!($($tail),*);
        paste::paste! {
            define_signature_trait!([< Sig $head >]$(, [< T $tail >])*);
        }
    };
}

mod signatures {
    define_signature_traits_for_arities!(10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
}

macro_rules! define_fn_traits_for_arities {
    () => {};
    ($head:tt$(, $tail:tt)*) => {
        define_fn_traits_for_arities!($($tail),*);

        paste::paste! {
            define_fn_trait!(Fn => [< Fn $head >] via [< Sig $head >]$(, [< T $tail >])*);
            define_fn_trait!(FnMut => [< FnMut $head >] via [< Sig $head >]$(, [< T $tail >])*);
            define_fn_trait!(FnOnce => [< FnOnce $head >] via [< Sig $head >]$(, [< T $tail >])*);
        }
    };
}

define_fn_traits_for_arities!(10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
