pub use paste;

#[macro_export]
macro_rules! protocol {
    (
        type Error = $err:ident;

        $(
            $module:ident {
                $(
                    fn $handler:ident ( $($arg:ident : $type:ty ),* ) -> $ret:ty;
                )*
            }
        )*
    ) => {
        purveyor::paste::paste! {
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

            #[allow(async_fn_in_trait)]
            pub trait Transport {
                async fn send(&self, request: Request) -> Result<Response, $err>;
            }

            $(
                pub struct [<$module:camel Client>]<'a, T: Transport> {
                    transport: &'a T,
                }

                impl<'a, T: Transport> [<$module:camel Client>]<'a, T> {
                    $(pub async fn $handler(&self, $($arg: $type),* ) -> Result<$ret, $err> {
                        if let Response::[<$module:camel $handler:camel>](out) = self.transport.send( Request::[<$module:camel $handler:camel>]( $($arg),* ) ).await? {
                            Ok(out)
                        } else {
                            unreachable!()
                        }
                    })*
                }
            )*

            pub struct Client<T: Transport> {
                transport: T,
            }

            impl<'a, T: Transport> Client<T> {
                pub fn new(transport: T) -> Self {
                    Self {
                        transport,
                    }
                }

                $(pub fn [<$module:snake>](&'a self) -> [<$module:camel Client>]<'a, T> {
                    [<$module:camel Client>] {
                        transport: &self.transport,
                    }
                })*
            }
        }
    };
}
