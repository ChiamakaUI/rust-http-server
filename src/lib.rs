pub mod schema;
pub mod handlers;
pub mod utils;

// Re-export commonly used items
pub use schema::{ApiResponse, KeypairResponse, InstructionResponse};
pub use handlers::create_routes;