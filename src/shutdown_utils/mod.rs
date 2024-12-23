use tokio;

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