pub use paste;

#[macro_export]
macro_rules! protocol {
    (
        type Error = $err:ty;

        $(
            $module:ident {
                $(
                    fn $handler:ident ( $($arg:ident : $type:ty ),* ) -> $ret:ty;
                )*
            }
        )*
    ) => {
        $crate::paste::paste! {
            // -----------------------------------------------------------------
            //                        Transport layer
            // -----------------------------------------------------------------

            #[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
            pub enum Request {
                $(
                    $([<$module:camel $handler:camel>]( $($type),* )),*
                ),*
            }

            #[derive(Debug, Clone, borsh::BorshSerialize, borsh::BorshDeserialize)]
            pub enum Response {
                $(
                    $([<$module:camel $handler:camel>]( $ret )),*
                ),*
            }

            #[allow(async_fn_in_trait)]
            pub trait Transport {
                async fn send(&self, request: Request) -> Result<Response, $err>;
            }

            // -----------------------------------------------------------------
            //                        Server traits
            // -----------------------------------------------------------------

            $(
                #[allow(async_fn_in_trait)]
                pub trait [<$module:camel Server>] {
                    type Error: Into<$err>;

                    $(async fn $handler(&self, $($arg: $type),* ) -> Result<$ret, Self::Error>;)*
                }
            )*


            pub trait Server: $([<$module:camel Server>]<Error = <Self as Server>::Error>+)* {
                type Error: Into<$err>;

                #[allow(async_fn_in_trait)]
                async fn receive(&self, request: Request) -> Result<Response, <Self as Server>::Error> {
                    Ok(match request {
                        $(
                            $(Request::[<$module:camel $handler:camel>]($($arg),*) => {
                                Response::[<$module:camel $handler:camel>](
                                [<$module:camel Server>]::$handler(
                                    self,
                                    $($arg),*
                                ).await?)
                            }),*
                        ),*
                    })
                }
            }

            // -----------------------------------------------------------------
            //                        Client traits
            // -----------------------------------------------------------------

            $(
                pub struct [<$module:camel Client>]<T: Transport> (pub T);

                impl<T: Transport> [<$module:camel Client>]<T> {
                    $(pub async fn $handler(&self, $($arg: $type),* ) -> Result<$ret, $err> {
                        if let Response::[<$module:camel $handler:camel>](out) = self.0.send( Request::[<$module:camel $handler:camel>]( $($arg),* ) ).await? {
                            Ok(out)
                        } else {
                            unreachable!()
                        }
                    })*
                }
            )*

            #[macro_export]
            macro_rules! __impl_modules {
                ($target:ty) => {
                    impl $target {
                        $(
                            pub fn [<$module:snake>](&self) -> [<$module:camel Client>]<&$target> {
                                [<$module:camel Client>](self)
                            }
                        )*
                    }
                }
            }
        }
    };
}
