#[macro_export]
macro_rules! str_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $id:ident {
            $( $x:ident => $y:expr ),*$(,)+
            $({
                $( $x_ext:ident ($($tname:ident: $t:ty $(,)?)+) => {$y_ext:expr, ($($p:tt)+) => $y_ext2:expr$(,)?} ),*$(,)+
            })?
        }
    ) => {
        $(#[$meta])*
        $vis enum $id {
            $(
                $x,
            )*
            $($(
                $x_ext($($t)+),
            )*)?
        }
        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(
                        Self::$x => $y.to_string(),
                    )*
                    $($(
                        Self::$x_ext($($tname,)+) => $y_ext,
                    )*)?
                })
            }
        }
        paste::paste! {
            $vis struct [<Parse $id Err>];
            impl std::str::FromStr for $id {
                type Err = [<Parse $id Err>];

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        $(
                            $y => Ok(Self::$x),
                        )*
                        $($(
                            $($p)+ => $y_ext2,
                        )*)?
                        _ => Err([<Parse $id Err>]),
                    }
                }
            }
        }
    };
}
