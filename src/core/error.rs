// =============================================================================
// REACTOR Error Handling System
// =============================================================================
// Unified error types for the entire engine.
// All errors flow through ReactorError for consistent handling.
// =============================================================================

use std::fmt;
use std::error::Error;

/// Error codes that can be returned through C ABI
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// No error
    None = 0,
    
    // Vulkan errors (100-199)
    /// Vulkan instance creation failed
    VulkanInstanceCreation = 100,
    /// Vulkan device creation failed
    VulkanDeviceCreation = 101,
    /// Vulkan surface creation failed
    VulkanSurfaceCreation = 102,
    /// Vulkan swapchain creation failed
    VulkanSwapchainCreation = 103,
    /// Vulkan render pass creation failed
    VulkanRenderPassCreation = 104,
    /// Vulkan pipeline creation failed
    VulkanPipelineCreation = 105,
    /// Vulkan buffer creation failed
    VulkanBufferCreation = 106,
    /// Vulkan image creation failed
    VulkanImageCreation = 107,
    /// Vulkan memory allocation failed
    VulkanMemoryAllocation = 108,
    /// Vulkan command buffer error
    VulkanCommandBuffer = 109,
    /// Vulkan synchronization error
    VulkanSynchronization = 110,
    /// Vulkan shader compilation failed
    VulkanShaderCompilation = 111,
    /// Vulkan descriptor set error
    VulkanDescriptorSet = 112,
    /// Vulkan validation layer error
    VulkanValidation = 113,
    
    // Resource errors (200-299)
    /// File not found
    FileNotFound = 200,
    /// Invalid file format
    InvalidFormat = 201,
    /// Texture loading failed
    TextureLoadFailed = 202,
    /// Model loading failed
    ModelLoadFailed = 203,
    /// Shader loading failed
    ShaderLoadFailed = 204,
    /// Asset not found in cache
    AssetNotFound = 205,
    
    // Window errors (300-399)
    /// Window creation failed
    WindowCreation = 300,
    /// Event loop error
    EventLoopError = 301,
    
    // System errors (400-499)
    /// Out of memory
    OutOfMemory = 400,
    /// Invalid parameter
    InvalidParameter = 401,
    /// Not initialized
    NotInitialized = 402,
    /// Already initialized
    AlreadyInitialized = 403,
    /// Operation not supported
    NotSupported = 404,
    /// Internal error
    InternalError = 405,
    
    // Scene errors (500-599)
    /// Invalid object index
    InvalidObjectIndex = 500,
    /// Invalid mesh handle
    InvalidMeshHandle = 501,
    /// Invalid material handle
    InvalidMaterialHandle = 502,
    
    /// Unknown error
    Unknown = 999,
}

impl ErrorCode {
    /// Get a human-readable description of the error code
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::None => "No error",
            ErrorCode::VulkanInstanceCreation => "Failed to create Vulkan instance",
            ErrorCode::VulkanDeviceCreation => "Failed to create Vulkan device",
            ErrorCode::VulkanSurfaceCreation => "Failed to create Vulkan surface",
            ErrorCode::VulkanSwapchainCreation => "Failed to create Vulkan swapchain",
            ErrorCode::VulkanRenderPassCreation => "Failed to create Vulkan render pass",
            ErrorCode::VulkanPipelineCreation => "Failed to create Vulkan pipeline",
            ErrorCode::VulkanBufferCreation => "Failed to create Vulkan buffer",
            ErrorCode::VulkanImageCreation => "Failed to create Vulkan image",
            ErrorCode::VulkanMemoryAllocation => "Failed to allocate Vulkan memory",
            ErrorCode::VulkanCommandBuffer => "Vulkan command buffer error",
            ErrorCode::VulkanSynchronization => "Vulkan synchronization error",
            ErrorCode::VulkanShaderCompilation => "Failed to compile shader",
            ErrorCode::VulkanDescriptorSet => "Vulkan descriptor set error",
            ErrorCode::VulkanValidation => "Vulkan validation layer error",
            ErrorCode::FileNotFound => "File not found",
            ErrorCode::InvalidFormat => "Invalid file format",
            ErrorCode::TextureLoadFailed => "Failed to load texture",
            ErrorCode::ModelLoadFailed => "Failed to load model",
            ErrorCode::ShaderLoadFailed => "Failed to load shader",
            ErrorCode::AssetNotFound => "Asset not found in cache",
            ErrorCode::WindowCreation => "Failed to create window",
            ErrorCode::EventLoopError => "Event loop error",
            ErrorCode::OutOfMemory => "Out of memory",
            ErrorCode::InvalidParameter => "Invalid parameter",
            ErrorCode::NotInitialized => "Not initialized",
            ErrorCode::AlreadyInitialized => "Already initialized",
            ErrorCode::NotSupported => "Operation not supported",
            ErrorCode::InternalError => "Internal error",
            ErrorCode::InvalidObjectIndex => "Invalid object index",
            ErrorCode::InvalidMeshHandle => "Invalid mesh handle",
            ErrorCode::InvalidMaterialHandle => "Invalid material handle",
            ErrorCode::Unknown => "Unknown error",
        }
    }
}

/// Main error type for REACTOR
#[derive(Debug)]
pub struct ReactorError {
    /// Error code for C ABI compatibility
    pub code: ErrorCode,
    /// Detailed error message
    pub message: String,
    /// Optional source error
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl ReactorError {
    /// Create a new error with code and message
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            source: None,
        }
    }
    
    /// Create an error with a source error
    pub fn with_source<E: Error + Send + Sync + 'static>(
        code: ErrorCode,
        message: impl Into<String>,
        source: E,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
    
    // Convenience constructors for common errors
    
    pub fn vulkan_instance(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VulkanInstanceCreation, msg)
    }
    
    pub fn vulkan_device(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VulkanDeviceCreation, msg)
    }
    
    pub fn vulkan_surface(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VulkanSurfaceCreation, msg)
    }
    
    pub fn vulkan_swapchain(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VulkanSwapchainCreation, msg)
    }
    
    pub fn vulkan_pipeline(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VulkanPipelineCreation, msg)
    }
    
    pub fn vulkan_buffer(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VulkanBufferCreation, msg)
    }
    
    pub fn vulkan_image(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VulkanImageCreation, msg)
    }
    
    pub fn vulkan_memory(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VulkanMemoryAllocation, msg)
    }
    
    pub fn vulkan_shader(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::VulkanShaderCompilation, msg)
    }
    
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::new(ErrorCode::FileNotFound, format!("File not found: {}", path.into()))
    }
    
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidFormat, msg)
    }
    
    pub fn invalid_parameter(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidParameter, msg)
    }
    
    pub fn not_initialized(what: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotInitialized, format!("{} not initialized", what.into()))
    }
    
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, msg)
    }
}

impl fmt::Display for ReactorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REACTOR-{:03}] {}: {}", 
            self.code as u32, 
            self.code.description(), 
            self.message
        )
    }
}

impl Error for ReactorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}

/// Result type alias for REACTOR operations
pub type ReactorResult<T> = Result<T, ReactorError>;

// =============================================================================
// Global Error State (for C ABI)
// =============================================================================

use std::sync::Mutex;

/// Global last error for C ABI access
static LAST_ERROR: Mutex<Option<ReactorError>> = Mutex::new(None);

/// Set the last error (called internally when errors occur)
pub fn set_last_error(error: ReactorError) {
    log::error!("{}", error);
    *LAST_ERROR.lock().unwrap() = Some(error);
}

/// Get the last error code
pub fn get_last_error_code() -> ErrorCode {
    LAST_ERROR.lock().unwrap()
        .as_ref()
        .map(|e| e.code)
        .unwrap_or(ErrorCode::None)
}

/// Get the last error message
pub fn get_last_error_message() -> Option<String> {
    LAST_ERROR.lock().unwrap()
        .as_ref()
        .map(|e| e.message.clone())
}

/// Clear the last error
pub fn clear_last_error() {
    *LAST_ERROR.lock().unwrap() = None;
}

/// Check if there's a pending error
pub fn has_error() -> bool {
    LAST_ERROR.lock().unwrap().is_some()
}

// =============================================================================
// Conversion traits for common error types
// =============================================================================

impl From<std::io::Error> for ReactorError {
    fn from(err: std::io::Error) -> Self {
        let code = match err.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::FileNotFound,
            std::io::ErrorKind::OutOfMemory => ErrorCode::OutOfMemory,
            _ => ErrorCode::InternalError,
        };
        ReactorError::with_source(code, err.to_string(), err)
    }
}

impl From<ash::vk::Result> for ReactorError {
    fn from(result: ash::vk::Result) -> Self {
        let code = match result {
            ash::vk::Result::ERROR_OUT_OF_HOST_MEMORY |
            ash::vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => ErrorCode::VulkanMemoryAllocation,
            ash::vk::Result::ERROR_INITIALIZATION_FAILED => ErrorCode::VulkanInstanceCreation,
            ash::vk::Result::ERROR_DEVICE_LOST => ErrorCode::VulkanDeviceCreation,
            ash::vk::Result::ERROR_SURFACE_LOST_KHR => ErrorCode::VulkanSurfaceCreation,
            _ => ErrorCode::InternalError,
        };
        ReactorError::new(code, format!("Vulkan error: {:?}", result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = ReactorError::new(ErrorCode::FileNotFound, "test.png");
        assert_eq!(err.code, ErrorCode::FileNotFound);
        assert!(err.message.contains("test.png"));
    }
    
    #[test]
    fn test_error_display() {
        let err = ReactorError::vulkan_pipeline("Invalid shader");
        let display = format!("{}", err);
        assert!(display.contains("105")); // ErrorCode::VulkanPipelineCreation
        assert!(display.contains("Invalid shader"));
    }
    
    #[test]
    fn test_global_error_state() {
        clear_last_error();
        assert!(!has_error());
        
        set_last_error(ReactorError::file_not_found("missing.obj"));
        assert!(has_error());
        assert_eq!(get_last_error_code(), ErrorCode::FileNotFound);
        
        clear_last_error();
        assert!(!has_error());
    }
}
