#[tokio::test]
async fn test_generator() {
    // -------------------------------------------------------------------------
    //                              Common code
    // -------------------------------------------------------------------------

    #[derive(Debug, PartialEq)]
    struct MyError {
        message: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct User {
        id: usize,
    }

    purveyor::protocol!(
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

    #[derive(Clone)]
    struct TestServer {}

    impl Server<MyError> for TestServer {}

    impl UserServer<MyError> for TestServer {
        async fn load(&self, id: usize) -> Result<User, MyError> {
            Ok(User { id })
        }

        async fn delete(&self, _id: usize) -> Result<(), MyError> {
            Ok(())
        }
    }

    impl ReportServer<MyError> for TestServer {
        async fn delete(&self, _id: usize) -> Result<(), MyError> {
            Err(MyError {
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

    impl Transport<MyError> for TestTransport {
        async fn send(&self, request: Request) -> Result<Response, MyError> {
            self.server.receive(request).await
        }
    }

    let client = Client::new(TestTransport {
        server: TestServer {},
    });

    assert_eq!(Ok(User { id: 1 }), client.user().load(1).await);

    assert_eq!(
        Err(MyError {
            message: String::from("Cannot delete")
        }),
        client.report().delete(1).await
    );
}
