use crate::{
    resp::{BulkString, Command, FromValue, SingleArgOrCollection, Value},
    BitmapCommands, ConnectionCommands, Future, GenericCommands, GeoCommands, HashCommands,
    HyperLogLogCommands, InnerClient, InternalPubSubCommands, IntoConfig, ListCommands, Pipeline,
    PreparedCommand, PubSubCommands, PubSubStream, Result, ScriptingCommands, SentinelCommands,
    ServerCommands, SetCommands, SortedSetCommands, StreamCommands, StringCommands, Transaction,
};
use std::future::IntoFuture;

/// A multiplexed client that can be cloned, allowing requests
/// to be be sent concurrently on the same underlying connection.
///
/// Compared to a [single client](crate::Client), a multiplexed client cannot offers access
/// to all existing Redis commands.
/// Transactions and [blocking commands](crate::BlockingCommands) are not compatible with a multiplexed client
/// because they monopolize the whole connection which cannot be shared anymore. It means other consumers of the same
/// multiplexed client will be blocked each time a transaction or a blocking command is in progress, losing the advantage
/// of a shared connection.
///
/// #See also [Multiplexing Explained](https://redis.com/blog/multiplexing-explained/)
#[derive(Clone)]
pub struct MultiplexedClient {
    inner_client: InnerClient,
}

impl MultiplexedClient {
    /// Connects asynchronously to the Redis server.
    ///
    /// # Errors
    /// Any Redis driver [`Error`](crate::Error) that occurs during the connection operation
    pub async fn connect(config: impl IntoConfig) -> Result<Self> {
        let inner_client = InnerClient::connect(config).await?;
        Ok(Self { inner_client })
    }

    /// Send an arbitrary command to the Redis server.
    ///
    /// This is used primarily intended for implementing high level commands API
    /// but may also be used to provide access to new features that lack a direct API.
    ///
    /// # Arguments
    /// * `name` - Command name in uppercase.
    /// * `args` - Command arguments which can be provided as arrays (up to 4 elements) or vectors of [`BulkString`](crate::resp::BulkString).
    ///
    /// # Errors
    /// Any Redis driver [`Error`](crate::Error) that occurs during the send operation
    ///
    /// # Example
    /// ```
    /// use redis_driver::{resp::cmd, MultiplexedClient, Result};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<()> {
    ///     let mut client = MultiplexedClient::connect("127.0.0.1:6379").await?;
    ///
    ///     let values: Vec<String> = client
    ///         .send(cmd("MGET").arg("key1").arg("key2").arg("key3").arg("key4"))
    ///         .await?
    ///         .into()?;
    ///     println!("{:?}", values);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn send(&mut self, command: Command) -> Result<Value> {
        self.inner_client.send(command).await
    }

    /// Send command to the Redis server and forget its response.
    ///
    /// # Errors
    /// Any Redis driver [`Error`](crate::Error) that occurs during the send operation
    pub fn send_and_forget(&mut self, command: Command) -> Result<()> {
        self.inner_client.send_and_forget(command)
    }

    /// Send a command batch to the Redis server.
    ///
    /// # Errors
    /// Any Redis driver [`Error`](crate::Error) that occurs during the send operation
    pub async fn send_batch(&mut self, commands: Vec<Command>) -> Result<Value> {
        self.inner_client.send_batch(commands).await
    }

    /// Create a new pipeline
    pub fn create_pipeline(&mut self) -> Pipeline {
        Pipeline::new(self.inner_client.clone())
    }

    /// Create a new transaction
    /// 
    /// Because of the multiplexed nature of the client,
    /// [`watch`](crate::TransactionCommands::watch) & 
    /// [`unwatch`](crate::TransactionCommands::unwatch)
    /// commands cannot be supported.
    /// To be able to use these commands with a transaction, 
    /// [`Client`](crate::Client) or [`PooledClientManager`](crate::PooledClientManager)
    /// should be used instead
    pub fn create_transaction(&mut self) -> Transaction {
        Transaction::new(self.inner_client.clone())
    }
}

pub trait MultiplexedPreparedCommand<'a, R>
where
    R: FromValue,
{
    /// Send command and forget its response
    ///
    /// # Errors
    /// Any Redis driver [`Error`](crate::Error) that occur during the send operation
    fn forget(self) -> Result<()>;
}

impl<'a, R> MultiplexedPreparedCommand<'a, R> for PreparedCommand<'a, MultiplexedClient, R>
where
    R: FromValue + Send + 'a,
{
    /// Send command and forget its response
    ///
    /// # Errors
    /// Any Redis driver [`Error`](crate::Error) that occur during the send operation
    fn forget(self) -> Result<()> {
        self.executor.send_and_forget(self.command)
    }
}

impl<'a, R> IntoFuture for PreparedCommand<'a, MultiplexedClient, R>
where
    R: FromValue + Send + 'a,
{
    type Output = Result<R>;
    type IntoFuture = Future<'a, R>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move { self.executor.send(self.command).await?.into() })
    }
}

impl BitmapCommands for MultiplexedClient {}
impl ConnectionCommands for MultiplexedClient {}
impl GenericCommands for MultiplexedClient {}
impl GeoCommands for MultiplexedClient {}
impl HashCommands for MultiplexedClient {}
impl HyperLogLogCommands for MultiplexedClient {}
impl InternalPubSubCommands for MultiplexedClient {}
impl ListCommands for MultiplexedClient {}
impl ScriptingCommands for MultiplexedClient {}
impl SentinelCommands for MultiplexedClient {}
impl ServerCommands for MultiplexedClient {}
impl SetCommands for MultiplexedClient {}
impl SortedSetCommands for MultiplexedClient {}
impl StreamCommands for MultiplexedClient {}
impl StringCommands for MultiplexedClient {}

impl PubSubCommands for MultiplexedClient {
    fn subscribe<'a, C, CC>(&'a mut self, channels: CC) -> Future<'a, PubSubStream>
    where
        C: Into<BulkString> + Send + 'a,
        CC: SingleArgOrCollection<C>,
    {
        self.inner_client.subscribe(channels)
    }

    fn psubscribe<'a, P, PP>(&'a mut self, patterns: PP) -> Future<'a, PubSubStream>
    where
        P: Into<BulkString> + Send + 'a,
        PP: SingleArgOrCollection<P>,
    {
        self.inner_client.psubscribe(patterns)
    }
}