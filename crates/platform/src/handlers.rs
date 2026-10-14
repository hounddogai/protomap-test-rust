//! The declarative endpoint macro used by domains that separate tonic adapters from workflow methods.

#[macro_export]
macro_rules! endpoints {
    ($contract:path, $handler:ty;
        unary { $( $unary:ident ($uq:ty) -> $ur:ty => $uw:ident; )* }
        server { $( $server:ident ($sq:ty) -> $sr:ty, $stream:ident => $sw:ident; )* }
        client { $( $client:ident ($cq:ty) -> $cr:ty => $cw:ident; )* }
        bidi { $( $bidi:ident ($bq:ty) -> $br:ty, $bstream:ident => $bw:ident; )* }
    ) => {
        #[tonic::async_trait]
        impl $contract for $handler {
            $(async fn $unary(&self, request: tonic::Request<$uq>)
                -> Result<tonic::Response<$ur>, tonic::Status>
            {
                self.$uw(request.into_inner()).await.map(tonic::Response::new)
            })*
            $(type $stream = tokio_stream::wrappers::ReceiverStream<Result<$sr, tonic::Status>>;
            async fn $server(&self, request: tonic::Request<$sq>)
                -> Result<tonic::Response<Self::$stream>, tonic::Status>
            {
                let handler = self.clone();
                let stream = $crate::transport::one_reply(async move {
                    handler.$sw(request.into_inner()).await
                });
                Ok(tonic::Response::new(stream))
            })*
            $(async fn $client(&self, request: tonic::Request<tonic::Streaming<$cq>>)
                -> Result<tonic::Response<$cr>, tonic::Status>
            {
                let mut stream = request.into_inner();
                let mut response = <$cr>::default();
                while let Some(message) = stream.message().await? {
                    response = self.$cw(message).await?;
                }
                Ok(tonic::Response::new(response))
            })*
            $(type $bstream = tokio_stream::wrappers::ReceiverStream<Result<$br, tonic::Status>>;
            async fn $bidi(&self, request: tonic::Request<tonic::Streaming<$bq>>)
                -> Result<tonic::Response<Self::$bstream>, tonic::Status>
            {
                let handler = self.clone();
                let mut incoming = request.into_inner();
                let (sender, receiver) = tokio::sync::mpsc::channel(16);
                tokio::spawn(async move {
                    loop {
                        let received = tokio::select! {
                            _ = sender.closed() => break,
                            received = incoming.message() => received,
                        };
                        let result = match received {
                            Ok(Some(message)) => handler.$bw(message).await,
                            Ok(None) => break,
                            Err(error) => Err(error),
                        };
                        let failed = result.is_err();
                        if sender.send(result).await.is_err() || failed { break; }
                    }
                });
                Ok(tonic::Response::new(tokio_stream::wrappers::ReceiverStream::new(receiver)))
            })*
        }
    };
}
