use hashbrown::HashSet;
use std::{borrow::Cow, convert::TryInto, marker::PhantomData, sync::Arc};

use specs::saveload::MarkerAllocator;
use wasmer::{imports, Cranelift, Function, Instance, Memory, Module, Store, Value, JIT};

use super::{
    errors::{PluginError, PluginModuleError},
    memory_manager::{self, EcsAccessManager, EcsWorld, MemoryManager},
    wasm_env::HostFunctionEnvironment,
};

use plugin_api::{
    raw::{RawAction, RawRequest, RawResponse},
    Event,
};

#[derive(Clone)]
/// This structure represent the WASM State of the plugin.
pub struct PluginModule {
    ecs: Arc<EcsAccessManager>,
    wasm_state: Instance,
    memory_manager: Arc<MemoryManager>,
    events: HashSet<String>,
    allocator: Function,
    memory: Memory,
    name: String,
}

impl PluginModule {
    /// This function takes bytes from a WASM File and compile them
    pub fn new(name: String, wasm_data: &[u8]) -> Result<Self, PluginModuleError> {
        // This is creating the engine is this case a JIT based on Cranelift
        let engine = JIT::new(Cranelift::default()).engine();
        // We are creating an enironnement
        let store = Store::new(&engine);
        // We are compiling the WASM file in the previously generated environement
        let module = Module::new(&store, &wasm_data).expect("Can't compile");

        // This is the function imported into the wasm environement
        fn raw_emit_actions(env: &HostFunctionEnvironment, ptr: i64, len: i64) {
            match env.read_data::<Vec<RawAction>>(from_i64(ptr), from_i64(len)) {
                Ok(actions) => {
                    for action in actions {
                        tracing::debug!("At this point, we should handle {:?}", action);
                    }
                },
                Err(e) => tracing::error!(?e, "Can't decode action"),
            }
        }

        fn raw_request(env: &HostFunctionEnvironment, ptr: i64, len: i64) -> i64 {
            let out = match env.read_data(from_i64(ptr), from_i64(len)) {
                Ok(data) => request(&env.ecs, data),
                Err(_) => Err(()),
            };

            // If an error happen set the i64 to 0 so the WASM side can tell an error
            // occured
            to_i64(env.write_data_as_pointer(&out).unwrap())
        }

        fn raw_print(env: &HostFunctionEnvironment, ptr: i64, len: i64) {
            if let Some(msg) = env
                .read_bytes(from_i64(ptr), from_i64(len))
                .and_then(|bytes| String::from_utf8(bytes).ok())
            {
                tracing::info!("[{}] {}", env.name, msg);
            } else {
                tracing::error!("Logging message from plugin {} failed!", env.name);
            }
        }

        let ecs = Arc::new(EcsAccessManager::default());
        let memory_manager = Arc::new(MemoryManager::default());

        // Create an import object.
        let import_object = imports! {
            "env" => {
                "raw_emit_actions" => Function::new_native_with_env(&store, HostFunctionEnvironment::new(name.clone(), ecs.clone(), memory_manager.clone()), raw_emit_actions),
                "raw_request" => Function::new_native_with_env(&store, HostFunctionEnvironment::new(name.clone(), ecs.clone(), memory_manager.clone()), raw_request),
                "raw_print" => Function::new_native_with_env(&store, HostFunctionEnvironment::new(name.clone(), ecs.clone(), memory_manager.clone()), raw_print),
            }
        };

        // Create an instance (Code execution environement)
        let instance = Instance::new(&module, &import_object)
            .map_err(PluginModuleError::InstantiationError)?;
        Ok(Self {
            memory_manager,
            ecs,
            memory: instance
                .exports
                .get_memory("memory")
                .map_err(PluginModuleError::MemoryUninit)?
                .clone(),
            allocator: instance
                .exports
                .get_function("wasm_prepare_buffer")
                .map_err(PluginModuleError::MemoryUninit)?
                .clone(),
            events: instance
                .exports
                .iter()
                .map(|(name, _)| name.to_string())
                .collect(),
            wasm_state: instance,
            name,
        })
    }

    /// This function tries to execute an event for the current module. Will
    /// return None if the event doesn't exists
    pub fn try_execute<'a, T>(
        &self,
        ecs: &EcsWorld,
        request: &PreparedEventQuery<T>,
    ) -> Option<Result<<T as Event<'a>>::Response, PluginModuleError>>
    where
        T: Event<'a>,
    {
        if !self.events.contains(request.function_name.as_ref()) {
            return None;
        }
        // Store the ECS Pointer for later use in `retreives`
        let bytes = match self.ecs.execute_with(ecs, || {
            execute_raw(
                self,
                &self.wasm_state,
                &request.function_name,
                &request.bytes,
            )
        }) {
            Ok(e) => e,
            Err(e) => return Some(Err(e)),
        };
        Some(bincode::deserialize(&bytes).map_err(PluginModuleError::Encoding))
    }
}

/// This structure represent a Pre-encoded event object (Useful to avoid
/// reencoding for each module in every plugin)
pub struct PreparedEventQuery<'a, T> {
    bytes: Vec<u8>,
    function_name: Cow<'a, str>,
    _phantom: PhantomData<T>,
}

impl<'a, T: Event<'a>> PreparedEventQuery<'a, T> {
    /// Create a prepared query from an event reference (Encode to bytes the
    /// struct) This Prepared Query is used by the `try_execute` method in
    /// `PluginModule`
    pub fn new(raw_event: &'a <T as Event<'a>>::Raw) -> Result<Self, PluginError> {
        Ok(Self {
            bytes: bincode::serialize(raw_event).map_err(PluginError::Encoding)?,
            function_name: T::get_handler_name(raw_event),
            _phantom: PhantomData::default(),
        })
    }

    pub fn get_function_name(&self) -> &str { &self.function_name }
}

/// This function split a u128 in two u64 encoding them as le bytes
pub fn from_u128(i: u128) -> (u64, u64) {
    let i = i.to_le_bytes();
    (
        u64::from_le_bytes(i[0..8].try_into().unwrap()),
        u64::from_le_bytes(i[8..16].try_into().unwrap()),
    )
}

/// This function merge two u64 encoded as le in one u128
pub fn to_u128(a: u64, b: u64) -> u128 {
    let a = a.to_le_bytes();
    let b = b.to_le_bytes();
    u128::from_le_bytes([a, b].concat().try_into().unwrap())
}

/// This function encode a u64 into a i64 using le bytes
pub fn to_i64(i: u64) -> i64 { i64::from_le_bytes(i.to_le_bytes()) }

/// This function decode a i64 into a u64 using le bytes
pub fn from_i64(i: i64) -> u64 { u64::from_le_bytes(i.to_le_bytes()) }

// This function is not public because this function should not be used without
// an interface to limit unsafe behaviours
#[allow(clippy::needless_range_loop)]
fn execute_raw(
    module: &PluginModule,
    instance: &Instance,
    event_name: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, PluginModuleError> {
    // This write into memory `bytes` using allocation if necessary returning a
    // pointer and a length

    let (mem_position, len) =
        module
            .memory_manager
            .write_bytes(&module.memory, &module.allocator, bytes)?;

    // This gets the event function from module exports

    let func = instance
        .exports
        .get_function(event_name)
        .map_err(PluginModuleError::MemoryUninit)?;

    // We call the function with the pointer and the length

    let function_result = func
        .call(&[Value::I64(to_i64(mem_position)), Value::I64(to_i64(len))])
        .map_err(PluginModuleError::RunFunction)?;

    // Waiting for `multi-value` to be added to LLVM. So we encode a pointer to a
    // u128 that represent [u64; 2]

    let u128_pointer = from_i64(
        function_result[0]
            .i64()
            .ok_or_else(PluginModuleError::InvalidArgumentType)?,
    );

    // TODO: Codify error
    let bytes = memory_manager::read_bytes(&module.memory, u128_pointer, 16).unwrap();

    // We read the return object and deserialize it

    // The first 8 bytes are encoded as le and represent the pointer to the data
    // The next 8 bytes are encoded as le and represent the length of the data
    // TODO: Codify error
    Ok(memory_manager::read_bytes(
        &module.memory,
        u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
        u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
    )
    .unwrap())
}

fn request<'a>(ecs: &'a EcsAccessManager, req: RawRequest) -> Result<RawResponse<'a>, ()> {
    match req {
        RawRequest::EntityName(e) => {
            // Safety: No reference is leaked out the function so it is safe.
            let world = unsafe {
                ecs.get().ok_or(
                    (), /* RetrieveError::EcsAccessError(EcsAccessError::EcsPointerNotAvailable) */
                )?
            };
            let entity = world.uid_allocator.retrieve_entity_internal(e.0).ok_or(
                (), /* RetrieveError::EcsAccessError(EcsAccessError::EcsEntityNotFound(e)) */
            )?;

            Ok(RawResponse::EntityName(Cow::Borrowed(
                &world
                    .player
                    .get(entity)
                    .ok_or_else(|| {
                        /*
                        RetrieveError::EcsAccessError(EcsAccessError::EcsComponentNotFound(
                            e,
                            "Player".to_owned(),
                        ))
                        */
                    })?
                    .alias,
            )))
        },
        RawRequest::EntityHealth(e) => {
            // Safety: No reference is leaked out the function so it is safe.
            let world = unsafe {
                ecs.get().ok_or(
                    (), /*RetrieveError::EcsAccessError(
                            EcsAccessError::EcsPointerNotAvailable,
                        )*/
                )?
            };
            let entity = world.uid_allocator.retrieve_entity_internal(e.0).ok_or(
                (), /*
                    RetrieveError::EcsAccessError(EcsAccessError::EcsEntityNotFound(e))*/
            )?;
            Ok(RawResponse::EntityHealth(
                *world.health.get(entity).ok_or_else(|| {
                    /*
                    RetrieveError::EcsAccessError(EcsAccessError::EcsComponentNotFound(
                        e,
                        "Health".to_owned(),
                    ))
                    */
                })?,
            ))
        },
        RawRequest::PlayerUidByName(_) => todo!(),
    }
}
