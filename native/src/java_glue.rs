use jni::{
    JNIEnv,
    objects::{JClass, JObject, JString, JValue},
    sys::jlong,
};

use crate::handler::CommandHandler;

#[repr(C)]
union StatePointer {
    ptr: *mut CommandHandler,
    _long: jlong,
}
impl StatePointer {
    fn new(ptr: *mut CommandHandler) -> Self {
        Self { ptr }
    }
    fn into_ptr(self) -> *mut CommandHandler {
        // SAFETY: This union is only accessed through `ptr`.
        unsafe { self.ptr }
    }
}
const _: () = assert!(size_of::<StatePointer>() == size_of::<jlong>());

#[unsafe(no_mangle)]
extern "system" fn Java_edu_kit_kastel_CommandHandler_initNative(
    _env: JNIEnv,
    _class: JClass,
) -> StatePointer {
    let boxed_state = Box::new(CommandHandler::new(crate::model::Model::new()));
    StatePointer::new(Box::into_raw(boxed_state))
}

#[unsafe(no_mangle)]
extern "system" fn Java_edu_kit_kastel_CommandHandler_cleanupNative(
    _env: JNIEnv,
    _class: JClass,
    state: StatePointer,
) {
    // SAFETY:
    // These methods are private and the state was initialized as a box.
    // This is the last place that sees the value of this pointer, as it is zeroed out
    // afterwards. We can safely reclaim and drop the box.
    let _ = unsafe { Box::from_raw(state.into_ptr()) };
}

#[unsafe(no_mangle)]
extern "system" fn Java_edu_kit_kastel_CommandHandler_handleCommandNative<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    state: StatePointer,
    command: JString<'local>,
) -> JObject<'local> {
    // SAFETY:
    // All methods accessing the command handler's native state are synchronized, so we have
    // exclusive access to the state for the duration of this method (which is the lifetime of the
    // reference). The state pointer is valid since it comes from a valid `Box`.
    let exclusive_reference: &mut _ = unsafe { &mut *state.into_ptr() };

    let command = env.get_string(&command).unwrap().into();

    let crate::handler::CommandResult {
        command_output,
        is_error,
        is_quitting: is_qutting,
    } = exclusive_reference.handle_command(command);

    let command_output = env.new_string(command_output).unwrap();

    env.new_object(
        "edu/kit/kastel/CommandResult",
        "(Ljava/lang/String;ZZ)V",
        &[
            JValue::from(&command_output),
            is_error.into(),
            is_qutting.into(),
        ],
    )
    .unwrap()
}
