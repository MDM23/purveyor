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

    #[derive(Clone)]
    struct TestServer {}

    impl Server for TestServer {
        type Error = InternalError;
    }

    impl UserServer for TestServer {
        type Error = InternalError;

        async fn load(&self, id: usize) -> Result<User, Self::Error> {
            Ok(User { id })
        }

        async fn delete(&self, _id: usize) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl ReportServer for TestServer {
        type Error = InternalError;

        async fn delete(&self, _id: usize) -> Result<(), Self::Error> {
            Err(InternalError {
                message: String::from("Cannot delete"),
            })
        }
    }

    // -------------------------------------------------------------------------
    //                              Client side
    // -------------------------------------------------------------------------

    #[derive(Clone)]
    struct TestTransport {
        server: TestServer,
    }

    impl Transport for TestTransport {
        async fn send(&self, request: Request) -> Result<Response, PublicError> {
            self.server.receive(request).await.map_err(Into::into)
        }
    }

    let client = Client::new(TestTransport {
        server: TestServer {},
    });

    assert_eq!(Ok(User { id: 1 }), client.user().load(1).await);

    assert_eq!(
        Err(PublicError {
            message: String::from("Cannot delete")
        }),
        client.report().delete(1).await
    );
}
