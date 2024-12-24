use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::process::Output;

use futures::future::Then;
use futures::FutureExt;
use tokio;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/**
 * Future which resolves on ctrl+c user input.
 * 
 * Used for server shutdown.
 */
pub async fn shutdown_on_ctrl_c(){
    tokio::signal::ctrl_c()
        .await
        .expect("Could not attach to ctrl-c listener");
}


/**
 * Helper struct to manage server shutdown.
 */
pub struct ShutdownHelper{
    active: Arc<AtomicU32>,
}

impl ShutdownHelper{
    /**
     * Create a new ShutdownHelper.
     */
    pub fn new() -> Self{
        Self{
            active: Arc::new(AtomicU32::new(0)),
        }
    }

    /**
     * Register a new connection.
     * 
     * This method returns a future which resolves when the connection is finished.
     * 
     * # Returns
     * A future which resolves when the connection is finished. 
     * 
     * # Example
     * ```
     * let shutdown_helper = ShutdownHelper::new();
     * let connection = shutdown_helper.register();
     * 
     * tokio::spawn(async move {
     *    // do some works
     *   connection.send(()).unwrap();
     * });
     * ```
     */
    pub fn register(&mut self) -> tokio::sync::oneshot::Sender<()>{
        let active = Arc::clone(&self.active);
        self.active.fetch_add(1, Ordering::SeqCst);

        let (tx, rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            rx.await.unwrap();
            active.fetch_sub(1, Ordering::SeqCst);
        });
        tx
    }
    /**
     * Shutdown the server.
     * 
     * This method waits for all active connections to finish before returning.
     */
    pub async fn shutdown(&self){
        while self.active.load(Ordering::SeqCst) > 0{
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
}