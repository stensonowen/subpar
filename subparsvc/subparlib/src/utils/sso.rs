//! Macro for newtypes of short strings on the stack.
//! The only validation done on such types is length checking.
//! Resulting type is POD.

use std::{error::Error as StdError, fmt};

#[derive(Debug, Clone)]
pub struct ShortStringOverflow {
    typename: &'static str,
    payload: String,
    source: Option<arrayvec::CapacityError>,
}

#[macro_export]
macro_rules! newt {

    (
        $( #[ $comments:meta ] ) *
        $v:vis
        struct
        $name:ident
        [ $array_length:literal ]
        $( ; )?

    ) => {

        $(#[$comments])* $v struct $name(
            ::arrayvec::ArrayString< $array_length >
        );

        impl $name {
            pub fn make(s: &str) -> Self {
                s.parse().unwrap()
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                f.write_str(self.0.as_str())
            }
        }

        impl core::str::FromStr for $name {
            type Err = crate::utils::sso::ShortStringOverflow;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                ::arrayvec::ArrayString::from(s).map( $name )
                    .map_err(|e| crate::utils::sso::ShortStringOverflow::with_source(
                            stringify!( $name ), s, e.simplify()))
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str { self.0.as_ref() }
        }
    };

}

/*
 * These hardcoded sizes correspond to trends in the NYCT system, and are not
 *  guaranteed to be followed either by other spec implementations
 *  nor by NYCT in the future.
 * May replace 'struct Train(ArrayString<9>)' with
 *  'enum Train { Short(ArrayString<9>), Long(String) }'
 *  so noncompliance only costs allocations without slowing the fast case.
 * Sizeof(String) == 24 I think
 *
 */

#[cfg(test)]
mod tests {
    use crate::newt;

    newt! {
        /// Basically 'struct Char(char);'
        #[derive(Clone, Debug, PartialEq, PartialOrd)]
        pub struct Char[1];
    }

    #[test]
    fn from_str() {
        let p = str::parse::<Char>;
        assert_eq!("A", p("A").unwrap().as_ref());
        assert_eq!(Char::make("A"), p("A").unwrap());
        assert!(p("AA").is_err());
    }

    #[test]
    #[ignore]
    fn _sample_of_what_type_err_macro_gen_err_unwrap_looks_like() {
        use std::str::FromStr;
        Char::from_str("AAA").unwrap();
    }

    // compile-time test
    fn _test_newtype_impls_partialord() {
        fn takes_partialord<T: PartialOrd>(_: T) {}
        fn _calls_partialord(new_type: Char) {
            takes_partialord(&new_type)
        }
    }
}

// impl ShortStringOverflow

impl StdError for ShortStringOverflow {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        if let Some(s) = &self.source {
            Some(s)
        } else {
            None
        }
    }
}

impl ShortStringOverflow {
    pub fn new(typename: &'static str, attempt: &str) -> Self {
        ShortStringOverflow {
            typename,
            payload: attempt.to_string(),
            source: None,
        }
    }
    pub fn with_source(
        typename: &'static str,
        attempt: &str,
        source: arrayvec::CapacityError,
    ) -> Self {
        ShortStringOverflow {
            typename,
            source: Some(source),
            payload: attempt.to_string(),
        }
    }
    pub fn typename(&self) -> &'static str {
        self.typename
    }
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

impl fmt::Display for ShortStringOverflow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "string too long for {} SSO type: '{}'",
            self.typename, self.payload
        )
    }
}
