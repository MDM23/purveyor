#[tokio::test]
async fn test_generator() {
    // -------------------------------------------------------------------------
    //                              Common code
    // -------------------------------------------------------------------------

    #[derive(Debug, PartialEq)]
    struct PublicError {
        message: String,
    }

    #[derive(Debug, Clone, PartialEq, borsh::BorshSerialize, borsh::BorshDeserialize)]
    struct User {
        id: usize,
    }

    purveyor::protocol!(
        type Error = PublicError;

        user {
            fn load(id: usize) -> User;
            fn delete(id: usize) -> ();
        }

        report {
            fn delete(id: usize) -> ();
        }
    );

    // -------------------------------------------------------------------------
    //                              Server side
    // -------------------------------------------------------------------------

    #[derive(Debug, PartialEq)]
    struct InternalError {
        message: String,
    }

    impl Into<PublicError> for InternalError {
        fn into(self) -> PublicError {
            PublicError {
                message: self.message,
            }
        }
    }

    struct MyServer {}

    impl Server for MyServer {
        type Error = InternalError;
    }

    impl UserServer for MyServer {
        type Error = InternalError;

        async fn load(&self, id: usize) -> Result<User, Self::Error> {
            Ok(User { id })
        }

        async fn delete(&self, _id: usize) -> Result<(), Self::Error> {
            unreachable!()
        }
    }

    impl ReportServer for MyServer {
        type Error = InternalError;

        async fn delete(&self, _id: usize) -> Result<(), Self::Error> {
            Err(InternalError {
                message: "Cannot delete".into(),
            })
        }
    }

    // -------------------------------------------------------------------------
    //                              Client side
    // -------------------------------------------------------------------------

    struct MyClient {
        server: MyServer,
    }

    impl Transport for &MyClient {
        async fn send(&self, request: Request) -> Result<Response, PublicError> {
            self.server.receive(request).await.map_err(Into::into)
        }
    }

    __impl_modules!(MyClient);

    let client = MyClient {
        server: MyServer {},
    };

    assert_eq!(Ok(User { id: 1 }), client.user().load(1).await);

    assert_eq!(
        Err(PublicError {
            message: String::from("Cannot delete")
        }),
        client.report().delete(1).await
    );
}
