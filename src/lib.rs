pub use paste;

#[macro_export]
macro_rules! protocol {
    (
        $(
            $module:ident {
                $(
                    fn $handler:ident ( $($arg:ident : $type:ty ),* ) -> $ret:ty;
                )*
            }
        )*
    ) => {
        purveyor::paste::paste! {
            #[derive(Debug, Clone)]
            #[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
            pub enum Request {
                $(
                    $([<$module:camel $handler:camel>]( $($type),* )),*
                ),*
            }

            #[derive(Debug, Clone)]
            #[cfg_attr(feature = "borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
            pub enum Response {
                $(
                    $([<$module:camel $handler:camel>]( $ret )),*
                ),*
            }

            $(
                #[allow(async_fn_in_trait)]
                pub trait [<$module:camel Server>]<ERROR> {
                    $(async fn $handler(&self, $($arg: $type),* ) -> Result<$ret, ERROR>;)*
                }
            )*

            pub trait Server<ERROR>: $([<$module:camel Server>]<ERROR>+)* {
                #[allow(async_fn_in_trait)]
                async fn receive(&self, request: Request) -> Result<Response, ERROR> {
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
            pub trait Transport<ERROR>: Clone {
                async fn send(&self, request: Request) -> Result<Response, ERROR>;
            }

            $(
                pub struct [<$module:camel Client>]<'a, T: Transport<ERROR>, ERROR> {
                    transport: &'a T,
                    _error: std::marker::PhantomData<ERROR>,
                }

                impl<'a, T: Transport<ERROR>, ERROR> [<$module:camel Client>]<'a, T, ERROR> {
                    $(async fn $handler(&self, $($arg: $type),* ) -> Result<$ret, ERROR> {
                        if let Response::[<$module:camel $handler:camel>](out) = self.transport.send( Request::[<$module:camel $handler:camel>]( $($arg),* ) ).await? {
                            Ok(out)
                        } else {
                            unreachable!()
                        }
                    })*
                }
            )*

            pub struct Client<T: Transport<ERROR>, ERROR> {
                transport: T,
                _error: std::marker::PhantomData<ERROR>,
            }

            impl<'a, T: Transport<ERROR>, ERROR> Client<T, ERROR> {
                pub fn new(transport: T) -> Self {
                    Self {
                        transport,
                        _error: std::marker::PhantomData,
                    }
                }

                $(pub fn [<$module:snake>](&'a self) -> [<$module:camel Client>]<'a, T, ERROR> {
                    [<$module:camel Client>] {
                        transport: &self.transport,
                        _error: self._error,
                    }
                })*
            }
        }
    };
}
